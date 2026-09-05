<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api, type DonationStatusResponse } from '../api/auth'
import { playSound } from '../sounds'
import { burstConfetti } from '../confetti'
import { applyTheme } from '../theme'
import SiteTopbar from '../components/SiteTopbar.vue'

const props = defineProps<{ id: string }>()
const route = useRoute()
const router = useRouter()

const qrUrl = computed(() => (route.query.qr as string) || '')
const payUrl = computed(() => (route.query.pay as string) || '')
const amount = computed(() => Number(route.query.amount ?? 0))
const soundKind = (route.query.sound as string) || 'chime'
const status = ref<DonationStatusResponse['status'] | null>(null)
let timer: ReturnType<typeof setInterval> | undefined

const secondsLeft = ref(15 * 60)

// เสียงพูดขอบคุณ (โหมด tts) — ลำดับเดียวกับ overlay: กระดิ่งนำ → เสียงอ่านตาม
function speakThanks() {
  const name = (route.query.name as string) || 'ผู้สนับสนุน'
  const text = `ขอบคุณ ${name} ที่โดเนต ${amount.value.toLocaleString()} บาท`
  const voices = speechSynthesis.getVoices()
  if (voices.length === 0) {
    // เครื่องไม่มี TTS voice — เล่นเสียงแตรวงแทน
    playSound('fanfare')
    return
  }
  const u = new SpeechSynthesisUtterance(text)
  u.lang = 'th-TH'
  const th = voices.find((v) => v.lang.startsWith('th'))
  if (th) u.voice = th
  speechSynthesis.cancel()
  speechSynthesis.speak(u)
}

onMounted(() => {
  applyTheme()
  timer = setInterval(async () => {
    secondsLeft.value = Math.max(0, secondsLeft.value - 1)
    try {
      const res = await api.getDonation(props.id)
      status.value = res.status
      if (res.status !== 'pending') {
        clearInterval(timer)
        // จ่ายสำเร็จ → เสียง + คอนเฟตติเฉลิมฉลองในแท็บนี้ด้วย
        if (res.status === 'paid') {
          if (soundKind === 'tts') {
            // กระดิ่งนำก่อน แล้วค่อยตามด้วยเสียงพูดขอบคุณ
            playSound('chime')
            setTimeout(speakThanks, 1400)
          } else {
            playSound(soundKind)
          }
          burstConfetti(80)
        }
        setTimeout(() => router.push('/'), 8000)
      }
    } catch {
      /* network สะดุดชั่วคราว — รอรอบถัดไป */
    }
  }, 1000)
})

onUnmounted(() => clearInterval(timer))

const countdown = computed(() => {
  const m = Math.floor(secondsLeft.value / 60)
  const s = secondsLeft.value % 60
  return `${m}:${String(s).padStart(2, '0')}`
})
</script>

<template>
  <div class="page">
    <SiteTopbar />
    <div class="page-body">
    <!-- กำลังรอชำระเงิน -->
    <div v-if="status === null || status === 'pending'" class="layout">
      <div class="card form-card">
        <div class="card-head">
          <h2>⏳ รอการชำระเงิน</h2>
          <span class="badge badge-pink">PromptPay</span>
        </div>
        <p class="sub">เปิดแอปธนาคาร → สแกน QR → ยืนยันยอดตามที่กรอกไว้</p>
        <ol class="steps">
          <li><b>Step 1</b> เปิดแอปธนาคารแล้วเลือกสแกน QR</li>
          <li><b>Step 2</b> เช็คยอดเงินให้ตรงกับที่กรอก</li>
          <li><b>Step 3</b> ยืนยันโอน — Alert จะเด้งในฉากสตรีมทันที</li>
        </ol>
        <button class="btn-ghost back" @click="router.push('/')">← ยกเลิกและกลับหน้าโดเนต</button>
      </div>

      <div class="card qr-card">
        <div class="card-head">
          <span class="badge badge-green">● เชื่อมต่อระบบสำเร็จ</span>
          <span class="qr-tag">Thai QR Payment</span>
        </div>
        <div class="qr-box">
          <div class="qr-head">PromptPay พร้อมเพย์</div>
          <div class="qr-body">
            <img v-if="qrUrl" :src="qrUrl" alt="PromptPay QR" class="qr" />
            <span class="qr-heart">❤️</span>
          </div>
          <div class="qr-amount">฿{{ amount.toLocaleString() }}.00</div>
        </div>

        <div class="expire-box">
          <b>⏳ QR หมดอายุในอีก {{ countdown }} นาที</b>
          <p>เมื่อระบบตรวจสอบรายการ สถานะจะเปลี่ยนเป็นสำเร็จ Alert จะเด้งในฉากสตรีมภายใน 1-3 วินาที</p>
        </div>

        <div class="actions">
          <a v-if="payUrl" :href="payUrl" target="_blank" class="btn-primary" style="text-decoration: none">
            🧪 จำลองการชำระเงิน
          </a>
          <button class="btn-ghost" @click="router.push('/')">↗ ยกเลิกลิงก์นี้</button>
        </div>
        <p class="hint">ระบบ mockup — ยังไม่มีการตัดเงินจริง กดปุ่มจำลองเพื่อทดสอบ Alert</p>
      </div>
    </div>

    <!-- จ่ายสำเร็จ -->
    <div v-else-if="status === 'paid'" class="single">
      <div class="card result success">
        <div class="big">🎉</div>
        <h1>โดเนตสำเร็จ!</h1>
        <p class="sub">ขอบคุณมาก ๆ ที่สนับสนุน ❤️ ไปดูป็อบอัพบนหน้าจอ stream ได้เลย</p>
        <div class="badge badge-green">Alert ถูกส่งแล้ว</div>
      </div>
    </div>

    <!-- จ่ายไม่สำเร็จ -->
    <div v-else class="single">
      <div class="card result failed">
        <div class="big">😢</div>
        <h1>ไม่สำเร็จ / หมดอายุ</h1>
        <p class="sub">ลองโดเนตใหม่อีกครั้งได้เลยนะ</p>
        <button class="btn-primary" @click="router.push('/')">← กลับไปโดเนตใหม่</button>
      </div>
    </div>
    </div>
  </div>
</template>

<style scoped>
.page { min-height: 100vh; background: var(--bg); display: flex; align-items: flex-start; justify-content: center; padding: 40px 20px; }
.layout { display: grid; grid-template-columns: 1fr 1fr; gap: 22px; width: 100%; max-width: 900px; align-items: start; }
.single { width: 100%; max-width: 480px; }
.page-body .layout, .page-body .single { display: grid; }
.card {
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius-xl); padding: 26px; box-shadow: var(--shadow-card);
}
.card-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
h2 { font-size: 17px; }
h1 { font-size: 26px; }
.sub { color: var(--text-dim); font-size: 14px; margin: 8px 0 18px; line-height: 1.6; }
.steps { list-style: none; display: grid; gap: 10px; }
.steps li {
  display: flex; align-items: center; gap: 10px; padding: 12px 14px;
  border-radius: var(--radius-md); border: 1px solid var(--border); background: var(--bg-input);
  font-size: 13.5px; color: var(--text-dim);
}
.steps b {
  padding: 3px 9px; border-radius: 7px; font-size: 11px;
  background: var(--emerald-soft); color: var(--emerald);
}
.back { margin-top: 18px; }
.qr-card { text-align: center; }
.qr-tag { font-size: 12px; color: var(--text-faint); font-weight: 600; }
.qr-box {
  max-width: 300px; margin: 4px auto 0; border-radius: var(--radius-lg); overflow: hidden;
  background: #fff; color: #0f172a; box-shadow: 0 8px 30px rgba(15, 23, 42, 0.25);
  border: 1px solid #e2e8f0;
}
.qr-head { background: #1e3a8a; color: #fff; font-size: 13.5px; font-weight: 700; padding: 10px; }
.qr-body { position: relative; padding: 18px; }
.qr { width: 210px; height: 210px; image-rendering: pixelated; }
.qr-heart {
  position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);
  width: 36px; height: 36px; border-radius: 50%;
  background: var(--primary); color: #fff; font-size: 16px;
  display: flex; align-items: center; justify-content: center;
  border: 3px solid #fff; box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}
.qr-amount { font-family: var(--font-head); font-size: 22px; font-weight: 800; padding: 6px 0 18px; }
.expire-box {
  margin: 16px auto 0; max-width: 340px; padding: 13px 15px; border-radius: var(--radius-md);
  background: rgba(251, 191, 36, 0.1); border: 1px solid rgba(251, 191, 36, 0.35);
}
.expire-box b { font-size: 13px; color: var(--gold); }
.expire-box p { font-size: 12px; color: var(--text-dim); margin-top: 5px; line-height: 1.6; }
.actions { display: flex; gap: 10px; justify-content: center; margin-top: 18px; }
.hint { margin-top: 12px; font-size: 11.5px; color: var(--text-faint); }
.result { text-align: center; padding: 44px 32px; }
.big { font-size: 64px; margin-bottom: 8px; }
.success { border-color: rgba(16, 185, 129, 0.4); }
.success h1 { color: var(--emerald); }
.failed { border-color: rgba(244, 63, 94, 0.4); }
.failed h1 { color: var(--primary); }
@media (max-width: 780px) { .layout { grid-template-columns: 1fr; } }
</style>
