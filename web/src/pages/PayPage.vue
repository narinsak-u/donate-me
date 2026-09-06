<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { applyTheme } from '../theme'
import { api } from '../api/auth'
import { useDonationWatch } from '../composables/useDonationWatch'
import SiteTopbar from '../components/SiteTopbar.vue'
import SlipUpload from '../components/SlipUpload.vue'

const props = defineProps<{ id: string }>()
const route = useRoute()
const router = useRouter()

// หน้า fallback ของ flow บนหน้าเดียว (ADR-0001) — สำหรับ refresh / ส่งลิงก์ข้ามเครื่อง
const qrUrl = computed(() => (route.query.qr as string) || '')
const payUrl = computed(() => (route.query.pay as string) || '')
const amount = computed(() => Number(route.query.amount ?? 0))
const soundKind = ((route.query.sound as string) || 'chime') as 'chime' | 'coin' | 'fanfare' | 'tts'
const donorName = (route.query.name as string) || 'ผู้สนับสนุน'

const { status, countdown, start } = useDonationWatch()
const reviewNote = ref('')
const mode = ref<'mock' | 'direct' | 'omise' | string>((route.query.mode as string) || '')

onMounted(() => {
  applyTheme()
  start(props.id, { sound: soundKind, name: donorName, amount: amount.value })
})

// โหมดไม่รู้จาก query (ลิงก์เก่า) → ถามสถานะครั้งเดียว
void api.getDonation(props.id).then((res) => (mode.value = res.mode)).catch(() => {})

// rejected → ดึงเหตุผล
watch(status, async (s) => {
  if (s === 'rejected') {
    try {
      const res = await api.getDonation(props.id)
      reviewNote.value = res.review_note
    } catch {
      /* เหตุผลไม่จำเป็น */
    }
  }
  // จ่ายสำเร็จ → ค้างไว้ให้อ่าน 8 วินาที
  if (s === 'paid') setTimeout(() => router.push('/'), 8000)
})
</script>

<template>
  <div class="page">
    <SiteTopbar />
    <main class="shell">
      <!-- รอชำระเงิน / รอตรวจสลิป -->
      <section v-if="status === null || status === 'pending' || status === 'awaiting_review'" class="card">
        <div class="card-head">
          <h2>สแกนเพื่อส่งกำลังใจ</h2>
          <span class="badge badge-pink">PromptPay</span>
        </div>
        <p class="sub">เปิดแอปธนาคาร → สแกน → ยืนยันยอด ฿{{ amount.toLocaleString() }}</p>

        <div class="qr-box">
          <div class="qr-head">PromptPay พร้อมเพย์</div>
          <div class="qr-body">
            <img v-if="qrUrl" :src="qrUrl" alt="PromptPay QR" class="qr" />
            <span class="qr-heart">❤️</span>
          </div>
          <div class="qr-amount">฿{{ amount.toLocaleString() }}.00</div>
        </div>

        <!-- โอนตรง: แนบสลิป -->
        <template v-if="mode === 'direct'">
          <SlipUpload v-if="status !== 'awaiting_review'" :donation-id="id" />
          <p v-if="status === 'awaiting_review'" class="review-note" role="status">
            🕒 ได้รับสลิปแล้ว — รอเจ้าของสตรีมตรวจสอบ หน้านี้จะเฉลิมฉลองทันทีที่อนุมัติ
          </p>
        </template>

        <div class="expire-box">
          <b>⏳ QR หมดอายุใน {{ countdown }} นาที</b>
          <p>
            {{ mode === 'direct'
              ? 'เงินโอนเข้าบัญชีของสตรีมเมอร์โดยตรง ไม่ผ่านเว็บเรา — โอนแล้วแนบสลิปได้เลย'
              : 'จ่ายสำเร็จเมื่อไหร่ หน้านี้จะเฉลิมฉลองให้ทันที และ Alert จะเด้งในฉากสตรีมภายในไม่กี่วินาที' }}
          </p>
        </div>

        <div class="actions">
          <a v-if="payUrl" :href="payUrl" target="_blank" class="btn-primary">🧪 จำลองการชำระเงิน</a>
          <button class="btn-ghost" @click="router.push('/')">← ยกเลิก</button>
        </div>
        <p class="hint">{{ mode === 'mock' ? 'ระบบจำลอง — ยังไม่มีการตัดเงินจริง' : 'โอนตรงเข้าบัญชีสตรีมเมอร์ ไม่มีค่าธรรมเนียมเพิ่ม' }}</p>
      </section>

      <!-- สำเร็จ -->
      <section v-else-if="status === 'paid'" class="card result success">
        <div class="big">🎉</div>
        <h2>โดเนตสำเร็จ!</h2>
        <p class="sub">ขอบคุณมาก ๆ ที่สนับสนุน ❤️ Alert เด้งบนฉากสตรีมแล้ว</p>
        <div class="badge badge-green">จ่ายเงินสำเร็จ</div>
      </section>

      <!-- สลิปถูกปฏิเสธ -->
      <section v-else-if="status === 'rejected'" class="card result failed">
        <div class="big">🚫</div>
        <h2>สลิปถูกปฏิเสธ</h2>
        <p v-if="reviewNote" class="sub">เหตุผล: {{ reviewNote }}</p>
        <p class="sub">ไม่มีการตัดเงินผ่านเว็บเรา — ลองโดเนตใหม่ได้เลย</p>
        <button class="btn-primary" @click="router.push('/')">← กลับไปโดเนตใหม่</button>
      </section>

      <!-- ไม่สำเร็จ / หมดอายุ -->
      <section v-else class="card result failed">
        <div class="big">⌛</div>
        <h2>QR หมดอายุหรือรายการไม่สำเร็จ</h2>
        <p class="sub">ไม่มีการตัดเงิน — ลองโดเนตใหม่ได้เลย</p>
        <button class="btn-primary" @click="router.push('/')">← กลับไปโดเนตใหม่</button>
      </section>
    </main>
  </div>
</template>

<style scoped>
.page { min-height: 100vh; }
.shell {
  max-width: 480px;
  margin: 0 auto;
  padding: 32px 20px 48px;
}
.card { padding: 26px; }
.card-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; }
h2 { font-size: 19px; }
.sub { color: var(--text-dim); font-size: 14px; margin: 8px 0 18px; line-height: 1.6; }
.qr-box {
  max-width: 300px;
  margin: 0 auto;
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: #fff;
  color: #0f172a;
  box-shadow: 0 8px 30px rgba(15, 23, 42, 0.25);
  border: 1px solid #e2e8f0;
}
.qr-head { background: #1e3a8a; color: #fff; font-size: 13.5px; font-weight: 700; padding: 10px; text-align: center; }
.qr-body { position: relative; padding: 18px; text-align: center; }
.qr { width: 210px; height: 210px; image-rendering: pixelated; }
.qr-heart {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--primary);
  color: #fff;
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 3px solid #fff;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}
.qr-amount { font-size: 22px; font-weight: 700; padding: 6px 0 18px; text-align: center; }
.review-note {
  margin-top: 14px;
  padding: 12px 14px;
  border-radius: var(--radius-md);
  background: rgba(16, 185, 129, 0.08);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: var(--emerald);
  font-size: 13px;
  text-align: center;
  line-height: 1.6;
}
.expire-box {
  margin: 16px auto 0;
  max-width: 380px;
  padding: 13px 15px;
  border-radius: var(--radius-md);
  background: var(--gold-soft);
  border: 1px solid rgba(251, 191, 36, 0.35);
  text-align: center;
}
.expire-box b { font-size: 13px; color: var(--gold); }
.expire-box p { font-size: 12px; color: var(--text-dim); margin-top: 5px; line-height: 1.6; }
.actions { display: flex; gap: 10px; justify-content: center; margin-top: 18px; flex-wrap: wrap; }
.actions .btn-primary { text-decoration: none; }
.hint { margin-top: 12px; font-size: 11.5px; color: var(--text-faint); text-align: center; }
.result { text-align: center; padding: 44px 28px; }
.big { font-size: 60px; margin-bottom: 8px; }
.success h2 { color: var(--emerald); }
.failed h2 { color: var(--primary); }
.failed .btn-primary { margin-top: 10px; }
</style>
