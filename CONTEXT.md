# Donate Me

แพลตฟอร์มรับโดเนตสำหรับสตรีมเมอร์ไทย — ผู้ชมส่งกำลังใจผ่าน PromptPay และ Alert จะเด้งบนฉากสตรีมแบบสด ๆ

## Language

**Streamer (สตรีมเมอร์)**:
เจ้าของหน้ารับโดเนต ระบุตัวตนด้วย username
_Avoid_: Owner, seller, ร้านค้า

**Donor (ผู้บริจาค)**:
คนที่ส่งเงินสนับสนุนสตรีมเมอร์ผ่านหน้าโดเนต
_Avoid_: Viewer, fan, customer

**Donation**:
รายการโดเนต 1 ครั้ง มีสถานะ pending → paid หรือ failed
_Avoid_: Transaction, order, payment

**Alert**:
ป็อปอัพที่เด้งบนฉากสตรีมเมื่อ Donation จ่ายสำเร็จ พร้อมเสียงประกอบ
_Avoid_: Notification, popup

**Overlay**:
หน้าเว็บแยกต่างหากสำหรับฝังใน OBS เพื่อแสดง Alert บนฉากสตรีม
_Avoid_: Widget, scene

**Streamer Directory (ทำเนียบสตรีมเมอร์)**:
หน้าแรกของเว็บ — รายชื่อ Streamer ทั้งหมดพร้อมสถิติ กดเข้าไปยังหน้าโดเนตของคนนั้น
_Avoid_: Home feed, landing page

**Demo Mode (ถอดออกแล้ว)**:
โหมดแสดงข้อมูลตัวอย่างในยุคที่หน้าโดเนตเข้าได้โดยไม่ระบุสตรีมเมอร์ — ปัจจุบันถอดออกทั้งหมด ห้ามแต่งข้อมูลปลอมบนหน้าจริงเด็ดขาด (ดู ADR-0002)
_Avoid_: Mock data, sample data

**PromptPay QR**:
คิวอาร์พร้อมเพย์ที่ผู้บริจาคสแกนเพื่อชำระเงินตามยอดที่กรอก
_Avoid_: Thai QR, payment QR

**Leaderboard**:
อันดับผู้บริจาคสูงสุดประจำเดือนของ Streamer หนึ่งคน
_Avoid_: Top donors, ranking

**Live Feed**:
รายการ Donation ล่าสุดที่แสดงสด ๆ บนหน้าโดเนต
_Avoid_: Recent activity, history

**Goal (เป้าหมาย)**:
ยอดเงินเป้าหมายที่ Streamer ตั้งไว้เพื่อแสดงแถบความคืบหน้าการระดมทุน
_Avoid_: Campaign, target
