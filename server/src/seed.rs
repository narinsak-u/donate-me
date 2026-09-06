//! Seed ข้อมูลตัวอย่างจริงลงฐานข้อมูล — ข้อมูลชุดนี้ "จริง" ใน DB ทุกจุด
//! (ต่างจาก mock data ที่เคยแต่งบนหน้าเว็บซึ่งถูกถอดออกแล้วตาม ADR-0002)
//!
//! รัน:   cargo run -- --seed           → ข้ามถ้ามีข้อมูลอยู่แล้ว
//!        cargo run -- --seed --force   → ล้างข้อมูลเดิมแล้ว seed ใหม่ทั้งหมด
//!
//! บัญชีสตรีมเมอร์ทุกตัวใช้รหัสผ่าน Test1234! (เข้า Creator Studio ได้จริง)

use rand::Rng;
use sqlx::sqlite::SqlitePool;

use crate::{auth, now_millis, random_id};

const SEED_PASSWORD: &str = "Test1234!";
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

struct SeedStreamer {
    username: &'static str,
    display_name: &'static str,
    goal: i64,
    tts: bool,
    leaderboard: bool,
    theme: &'static str,
    donations: usize,
}

const STREAMERS: &[SeedStreamer] = &[
    SeedStreamer { username: "peachgaming", display_name: "พีชแกมมิ่ง", goal: 35000, tts: true, leaderboard: true, theme: "pink", donations: 80 },
    SeedStreamer { username: "mewcraft", display_name: "มิ้วคราฟต์", goal: 18000, tts: true, leaderboard: true, theme: "green", donations: 60 },
    SeedStreamer { username: "ninzastar", display_name: "นินจาสตาร์", goal: 10000, tts: false, leaderboard: true, theme: "blue", donations: 45 },
    SeedStreamer { username: "fahsaiplay", display_name: "ฟ้าใสเพลย์", goal: 16000, tts: true, leaderboard: true, theme: "pink", donations: 35 },
    SeedStreamer { username: "bankoverload", display_name: "แบงค์โอเวอร์โหลด", goal: 0, tts: false, leaderboard: true, theme: "dark", donations: 28 },
    SeedStreamer { username: "kaitom99", display_name: "ไก่ต้ม 99", goal: 3500, tts: true, leaderboard: true, theme: "green", donations: 22 },
    SeedStreamer { username: "pluemch", display_name: "ปลื้มช์", goal: 8000, tts: true, leaderboard: false, theme: "blue", donations: 40 },
    SeedStreamer { username: "gunzgg", display_name: "กันซ์ GG", goal: 26000, tts: true, leaderboard: true, theme: "pink", donations: 70 },
];

const DONOR_NAMES: &[&str] = &[
    "Gunz_lnwZa", "ฟั่น HEARTROCKER", "MewSuppasit", "น้องเนยคัด", "Bank_Overload",
    "Cyber_Kaitom", "Pluem_Ch", "คุณปริสนธ์สปอนเซอร์", "มาม่าปิ้งย่าง", "NongSomZa_77",
    "Petchzaa", "ตัวเล็กใจใหญ่", "มือปราบมูฟออน", "Ice_BearTH", "แป้งโรยหน้า",
    "Kittipong_Gamer", "น้องวาเนน", "สายเปียสายซิ่ง", "Boat_Drifter", "พิมพ์ใจจุ๊บๆ",
    "ZeroTwo_Fan", "หมูหยองกรอบ", "Praewa_uzz", "ตาลใหญ่ใจดี",
];

const MESSAGES: &[&str] = &[
    "สู้ ๆ นะพี่ วันนี้เล่นเก่งมาก! 🔥",
    "เว็บทำดีมากครับ ขอแสดงเป็นกำลังใจ 💗",
    "กราฟวิ่งไหว อุ่นใจมาก 5555",
    "พี่คะเล่น Valorant ด้วยวาเนนะ สู้สู้ปุย!",
    "มาแล้วครับกำลังใจยามดึก 😴",
    "เพิ่งเที่ยงเองนะครับพี่ กินข้าวยัง!",
    "ปั้นไมค์ใหม่ถึงเป้าเร็ว ๆ นะคะ 🎙️",
    "ดูสตรีมทุกวันเลยครับ รักพี่มาก 💜",
    "เสียงอ่านไทยน่ารักมาก 555",
    "จ่ายแล้วนะคะ ขอบคุณที่สนุกทุกวัน",
    "วันนี้วานตายเยอะเลย ไม่เป็นไรนะพี่ 😂",
    "โดนน้องน้องส่งกำลังใจครับ เก่งอยู่แล้ว!",
    "มาสนับสนุนคนไทยด้วยกันครับ 🇹🇭",
    "ขอเพลงหน่อยครับ ขอเพลงเศร้า 🎵",
    "แร้นดอมโดนแดกตลอดเลยพี่ ฮามาก",
    "เฝ้าสตรีมตั้งแต่เปิดจนปิดเลยครับ 🌟",
    "เก็บเงินมาโดเนตจริง ๆ ครับ สู้ ๆ",
    "คอมเครื่องใหม่กำลังจะมา อดทนนะพี่ 💪",
];

const SOUNDS: &[&str] = &["chime", "coin", "fanfare"];

/// รัน seed — คืน Ok เสมอ (พิมพ์สถานะกลาย ๆ)
pub async fn run(db: &SqlitePool, force: bool) {
    let (count,): (i64,) =
        match sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(db).await {
            Ok(r) => r,
            Err(e) => return eprintln!("❌ seed: อ่านตาราง users ไม่ได้: {e}"),
        };
    if count > 0 && !force {
        println!("⏭️  ฐานข้อมูลมีข้อมูลอยู่แล้ว ({count} บัญชี) — ข้าม seed (ต้องการ seed ใหม่ทั้งหมด: cargo run -- --seed --force)");
        return;
    }
    if force {
        // ล้างเฉพาะข้อมูลโดเนต/สตรีมเมอร์ — settings ผูกกับ user จึงลบตาม
        for table in ["donations", "settings", "users"] {
            if let Err(e) = sqlx::query(&format!("DELETE FROM {table}")).execute(db).await {
                return eprintln!("❌ seed: ล้าง {table} ไม่สำเร็จ: {e}");
            }
        }
        println!("🧹 ล้างข้อมูลเดิมแล้ว");
    }

    let password_hash = match auth::hash_password(SEED_PASSWORD) {
        Ok(h) => h,
        Err(e) => return eprintln!("❌ seed: hash รหัสผ่านล้มเหลว: {e}"),
    };
    let now = now_millis();
    let mut rng = rand::thread_rng();
    let mut total_donations = 0i64;

    for s in STREAMERS {
        let uid = random_id();
        let created_at = now - 60 * DAY_MS;
        if let Err(e) = sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, display_name, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&uid)
        .bind(s.username)
        .bind(format!("{}@seed.donateme", s.username))
        .bind(&password_hash)
        .bind(s.display_name)
        .bind(created_at)
        .execute(db)
        .await
        {
            return eprintln!("❌ seed: สร้างบัญชี {} ไม่สำเร็จ: {e}", s.username);
        }

        if let Err(e) = sqlx::query(
            "INSERT INTO settings (user_id, goal_amount, tts_enabled, show_leaderboard, theme, alert_position)
             VALUES (?, ?, ?, ?, ?, 'middle')",
        )
        .bind(&uid)
        .bind(s.goal)
        .bind(s.tts as i64)
        .bind(s.leaderboard as i64)
        .bind(s.theme)
        .execute(db)
        .await
        {
            return eprintln!("❌ seed: ตั้งค่า {} ไม่สำเร็จ: {e}", s.username);
        }

        for j in 0..s.donations {
            // กระจายย้อนหลัง 30 วัน — ยิ่งเก่ายิ่งเบาบาง
            let age_ms: i64 =
                (rng.gen::<f64>().powf(0.6) * 30.0 * DAY_MS as f64).round() as i64;
            let created = now - age_ms.max(60_000);
            // 5% ทิ้งเป็น pending ล่าสุด (ยังไม่จ่าย), ที่เหลือจ่ายแล้ว/บางส่วน expired
            let status = if j < 2 && age_ms < DAY_MS {
                "pending"
            } else if rng.gen::<f64>() < 0.05 {
                "expired"
            } else {
                "paid"
            };
            let amount = if rng.gen::<f64>() < 0.03 {
                rng.gen_range(2000..=5000) // Super Chat หายาก
            } else {
                [20, 50, 100, 200, 500][rng.gen_range(0..5)] + rng.gen_range(0..2) * 10
            };
            let message: &str = if rng.gen::<f64>() < 0.35 {
                ""
            } else {
                MESSAGES[rng.gen_range(0..MESSAGES.len())]
            };
            let paid_at = if status == "paid" { Some(created + rng.gen_range(5_000..90_000)) } else { None };

            if let Err(e) = sqlx::query(
                "INSERT INTO donations (id, user_id, donor_name, amount, message, sound, status, payment_ref, created_at, paid_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(random_id())
            .bind(&uid)
            .bind(DONOR_NAMES[rng.gen_range(0..DONOR_NAMES.len())])
            .bind(amount)
            .bind(message)
            .bind(SOUNDS[rng.gen_range(0..SOUNDS.len())])
            .bind(status)
            .bind(format!("seed_{}", random_id()))
            .bind(created)
            .bind(paid_at)
            .execute(db)
            .await
            {
                return eprintln!("❌ seed: ใส่โดเนตของ {} ไม่สำเร็จ: {e}", s.username);
            }
            if status == "paid" {
                total_donations += 1;
            }
        }
        println!("  ✅ @{} ({} โดเนต, เป้า ฿{})", s.username, s.donations, s.goal);
    }

    println!("🎉 seed เสร็จ: สตรีมเมอร์ {} คน, โดเนตจ่ายแล้ว {total_donations} รายการ", STREAMERS.len());
    println!("🔑 รหัสผ่านบัญชี seed ทุกตัว: {SEED_PASSWORD}");
}
