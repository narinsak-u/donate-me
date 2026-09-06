//! Phase 7: PromptPay EMVCo QR — สร้าง payload จริงตามสเปคมาตรฐาน (มิใช่ mock)
//! ให้ผู้ชมสแกนด้วยแอปธนาคารใดก็ได้ แล้วโอนเข้าบัญชีสตรีมเมอร์ "โดยตรง"
//!
//! โครงสร้าง payload = TLV (tag 2 หลัก + length 2 หลัก + value):
//!   00 พิมพ์เขียวรูปแบบ = 01
//!   01 point-of-initiation = 12 (dynamic: มียอดกำหนดไว้)
//!   29 merchant account info:
//!     00 AID = A000000677010111 (PromptPay)
//!     01 เบอร์โทร (0066+9 หลักไม่มี 0 นำ) หรือเลขบัตร 13 หลัก
//!   53 สกุลเงิน = 764 (THB)
//!   54 ยอดเงิน (2 ตำแหน่งทศนิยม)
//!   58 ประเทศ = TH
//!   63 CRC16-CCITT (poly 0x1021, init 0xFFFF) ของทั้งข้อความรวม "6304"

/// เตรียม promptpay_id ให้เป็นรูปแบบที่สเปคต้องการ
/// - เบอร์มือถือ 10 หลัก (0XXXXXXXXX) → 0066 + 9 หลักหลังตัด 0
/// - เลขบัตรประชาชน 13 หลัก → ใช้ตรง ๆ
/// คืน None ถ้ารูปแบบไม่ถูกต้อง
pub fn normalize_id(raw: &str) -> Option<String> {
    let id: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    match id.len() {
        10 if id.starts_with('0') => Some(format!("0066{}", &id[1..])),
        13 => Some(id),
        // กรอกมาเป็น 66XXXXXXXXX แล้ว
        11 if id.starts_with("66") => Some(format!("00{id}")),
        _ => None,
    }
}

/// ตรวจว่า streamer ตั้งค่าพร้อมเพย์รับโอนตรงได้หรือยัง
pub fn is_valid_id(raw: &str) -> bool {
    !raw.is_empty() && normalize_id(raw).is_some()
}

fn tlv(tag: &str, value: &str) -> String {
    format!("{tag}{:02}{}", value.chars().count(), value)
}

/// CRC-16/CCITT-FALSE — poly 0x1021, init 0xFFFF (เกณฑ์พิสูจน์: "123456789" → 0x29B1)
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 };
        }
    }
    crc
}

/// สร้าง payload เต็ม (รวม CRC) — promptpay_id ต้องผ่าน normalize_id แล้ว
pub fn payload(normalized_id: &str, amount_satang: i64) -> String {
    let merchant = tlv("00", "A000000677010111") + &tlv("01", normalized_id);
    let mut body = String::new();
    body.push_str(&tlv("00", "01")); // payload format
    body.push_str(&tlv("01", "12")); // dynamic (ฝังยอด)
    body.push_str(&tlv("29", &merchant));
    body.push_str(&tlv("53", "764"));
    body.push_str(&tlv("54", &format!("{:.2}", amount_satang as f64 / 100.0)));
    body.push_str(&tlv("58", "TH"));
    let crc_target = format!("{body}6304");
    let crc = crc16(crc_target.as_bytes());
    format!("{crc_target}{crc:04X}")
}

/// เรนเดอร์ payload เป็น PNG (แบบเดียวกับ mock_qr)
pub fn qr_png(payload: &str) -> Result<Vec<u8>, String> {
    let code = qrcode::QrCode::new(payload).map_err(|e| format!("qr: {e}"))?;
    let image = code
        .render::<image::Luma<u8>>()
        .min_dimensions(320, 320)
        .dark_color(image::Luma([15u8]))
        .light_color(image::Luma([255u8]))
        .build();
    let mut png = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| format!("png: {e}"))?;
    Ok(png.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc16_reference() {
        // เกณฑ์มาตรฐาน CRC-16/CCITT-FALSE (check value จาก catalog)
        assert_eq!(crc16(b"123456789"), 0x29B1);
    }

    #[test]
    fn normalize_phone_and_citizen() {
        assert_eq!(normalize_id("0812345678").as_deref(), Some("0066812345678"));
        assert_eq!(normalize_id("081-234-5678").as_deref(), Some("0066812345678"));
        assert_eq!(normalize_id("3100501234567").as_deref(), Some("3100501234567"));
        assert_eq!(normalize_id("1234"), None);
        assert_eq!(normalize_id(""), None);
    }

    #[test]
    fn payload_shape() {
        let p = payload("0066812345678", 2000); // ฿20.00
        assert!(p.starts_with("000201010212")); // รูปแบบ 01 + dynamic 12 ตามสเปค
        assert!(p.contains("A000000677010111"));
        assert!(p.contains("5303764"));   // สกุลเงิน THB
        assert!(p.contains("540520.00")); // ยอด
        assert!(p.contains("5802TH"));
        assert!(p.contains("6304")); // CRC tag อยู่ก่อนค่า CRC 4 ตัวท้าย
        // CRC ท้าย payload ต้องผ่านการคำนวณซ้ำ (payload ตัด 4 ตัวท้าย = body + tag 6304 แล้ว)
        let (crc_target, crc) = p.split_at(p.len() - 4);
        let expect = crc16(crc_target.as_bytes());
        assert_eq!(crc, format!("{expect:04X}"));
    }
}
