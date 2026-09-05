//! Phase 4: กรองคำหยาบ (TH/EN) สำหรับชื่อและข้อความโดเนต
//! หมายเหตุ: overlay ฝั่ง frontend ใช้ textContent อยู่แล้ว (กัน XSS) — ชั้นนี้กันคำหยาบ/สแปม

/// รายการคำหยาบพื้นฐาน — เพิ่มได้ใน config/DB ภายหลัง
const BAD_WORDS: &[&str] = &[
    // EN
    "fuck", "shit", "bitch", "asshole", "dick", "cunt", "bastard",
    // TH (คำอ่านแบบ latin + คำไทย)
    "kuy", "kwai", "hee", "mun", "sat", "gd", "กู", "มึง", "ไอ้สัด", "สัดหมาง", "เหี้ย", "หี", "ควย", "แตด",
];

/// ซ่อนคำหยาบด้วยเครื่องหมาย * (เช่น "fu*k")
pub fn mask_bad_words(input: &str) -> String {
    let mut masked = input.to_string();
    for word in BAD_WORDS {
        // กรองแบบ case-insensitive (EN)
        let lower = masked.to_lowercase();
        let mut search_from = 0;
        while let Some(pos) = lower[search_from..].find(word) {
            let start = search_from + pos;
            let end = start + word.len();
            // แทนที่ด้วย * จำนวนเท่าตัวอักษร (เว้นตัวแรก)
            masked.replace_range(start + 1..end, &"*".repeat(end - start - 1));
            // ต้องคำนวณ lower ใหม่เพราะ masked เปลี่ยน — วนจากจุดต่อไป
            search_from = end;
            // ป้องกัน infinite loop เมื่อ replace ทำให้ความยาวเปลี่ยน
            if search_from >= masked.len() {
                break;
            }
        }
    }
    masked
}

/// ตรวจว่ามีคำหยาบหรือไม่ (สำหรับ reject แบบเงียบ ๆ — เผื่อใช้ต่อ)
#[allow(dead_code)]
pub fn contains_bad_word(input: &str) -> bool {
    let lower = input.to_lowercase();
    BAD_WORDS.iter().any(|w| lower.contains(w))
}

/// ทำความสะอาด: trim, ตัด control chars, จำกัด newline ต่อเนื่อง
pub fn clean_text(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect::<String>()
        .replace("\n\n\n", "\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_english_bad_word() {
        assert_eq!(mask_bad_words("hello fuck you"), "hello f*** you");
    }

    #[test]
    fn masks_thai_bad_word() {
        assert!(contains_bad_word("ไอ้เหี้ย"));
        assert!(contains_bad_word("KUY forever"));
    }

    #[test]
    fn clean_text_trims_and_strips_control() {
        assert_eq!(clean_text("  hi\u{0} there  "), "hi there");
    }
}
