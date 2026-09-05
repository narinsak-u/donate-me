<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api, type DonationStatusResponse } from '../api/auth'

const props = defineProps<{ id: string }>()
const route = useRoute()
const router = useRouter()

const qrUrl = computed(() => (route.query.qr as string) || '')
const payUrl = computed(() => (route.query.pay as string) || '')
const status = ref<DonationStatusResponse['status'] | null>(null)
let timer: ReturnType<typeof setInterval> | undefined

// QR อายุ 15 นาที
const secondsLeft = ref(15 * 60)

onMounted(() => {
  timer = setInterval(async () => {
    secondsLeft.value = Math.max(0, secondsLeft.value - 1)
    try {
      const res = await api.getDonation(props.id)
      status.value = res.status
      if (res.status !== 'pending') {
        clearInterval(timer)
        setTimeout(() => router.push('/'), 8000)
      }
    } catch {
      /* network สะดุดชั่วคราว — รอรอบถัดไป */
    }
  }, 2000)
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
    <div class="aurora" />
    <div class="stars" />

    <!-- กำลังรอชำระเงิน -->
    <div v-if="status === null || status === 'pending'" class="card">
      <div class="badge">⏳ รอการชำระเงิน</div>
      <h1>สแกนจ่ายผ่าน PromptPay</h1>
      <p class="subtitle">เปิดแอปธนาคาร → สแกน QR → ยืนยันยอดตามที่กรอกไว้</p>

      <div class="qr-frame">
        <img v-if="qrUrl" :src="qrUrl" alt="PromptPay QR" class="qr" />
        <div class="qr-corner tl" /><div class="qr-corner tr" /><div class="qr-corner bl" /><div class="qr-corner br" />
      </div>

      <p class="countdown">QR หมดอายุใน <strong>{{ countdown }}</strong></p>

      <a v-if="payUrl" :href="payUrl" target="_blank" class="mock-btn">
        🧪 จำลองการชำระเงิน (เปิดหน้า Mock Bank)
      </a>
      <p class="hint">ระบบ mockup — ยังไม่มีการตัดเงินจริง กดปุ่มด้านบนเพื่อจำลองว่าจ่ายแล้ว</p>
    </div>

    <!-- จ่ายสำเร็จ -->
    <div v-else-if="status === 'paid'" class="card success">
      <div class="big bounce">🎉</div>
      <h1>โดเนตสำเร็จ!</h1>
      <p class="subtitle">ขอบคุณมาก ๆ ที่สนับสนุน ❤️ ไปดูป็อบอัพบนหน้าจอ stream ได้เลย</p>
    </div>

    <!-- จ่ายไม่สำเร็จ -->
    <div v-else class="card failed">
      <div class="big">😢</div>
      <h1>ไม่สำเร็จ / หมดอายุ</h1>
      <p class="subtitle">ลองโดเนตใหม่อีกครั้งได้เลยนะ</p>
      <button class="retry" @click="router.push('/')">← กลับไปโดเนตใหม่</button>
    </div>
  </div>
</template>

<style scoped>
.page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  position: relative;
  background: linear-gradient(160deg, var(--bg-0) 0%, var(--bg-1) 45%, #0f3460 100%);
}
.card {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 430px;
  text-align: center;
  background: var(--glass);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 38px 34px;
  box-shadow: var(--shadow-card);
}
.badge {
  display: inline-block;
  background: rgba(255, 224, 102, 0.14);
  border: 1px solid rgba(255, 224, 102, 0.35);
  color: var(--gold);
  font-size: 12.5px;
  font-weight: 600;
  padding: 6px 16px;
  border-radius: 999px;
  margin-bottom: 16px;
}
h1 {
  font-size: 25px;
  font-weight: 700;
}
.subtitle {
  color: var(--text-dim);
  font-size: 14px;
  margin: 8px 0 26px;
}
.qr-frame {
  position: relative;
  display: inline-block;
  padding: 14px;
  background: #fff;
  border-radius: 20px;
  box-shadow: 0 0 50px rgba(255, 255, 255, 0.12);
}
.qr {
  width: 224px;
  height: 224px;
  display: block;
  image-rendering: pixelated;
}
.qr-corner {
  position: absolute;
  width: 22px;
  height: 22px;
  border: 3px solid var(--accent-1);
}
.tl { top: -3px; left: -3px; border-right: none; border-bottom: none; border-radius: 10px 0 0 0; }
.tr { top: -3px; right: -3px; border-left: none; border-bottom: none; border-radius: 0 10px 0 0; }
.bl { bottom: -3px; left: -3px; border-right: none; border-top: none; border-radius: 0 0 0 10px; }
.br { bottom: -3px; right: -3px; border-left: none; border-top: none; border-radius: 0 0 10px 0; }
.countdown {
  margin-top: 18px;
  font-size: 14px;
  color: var(--text-dim);
  font-variant-numeric: tabular-nums;
}
.countdown strong {
  color: var(--gold);
}
.mock-btn {
  display: block;
  margin-top: 22px;
  padding: 14px;
  border-radius: var(--radius-md);
  background: linear-gradient(135deg, var(--accent-1), var(--accent-2));
  color: #fff;
  font-weight: 700;
  text-decoration: none;
  font-size: 15px;
  box-shadow: 0 8px 26px rgba(255, 110, 199, 0.3);
  transition: transform 0.15s;
}
.mock-btn:hover {
  transform: translateY(-2px);
}
.hint {
  margin-top: 14px;
  font-size: 12px;
  color: var(--text-faint);
}
.big {
  font-size: 68px;
}
.bounce {
  animation: pop 0.6s cubic-bezier(0.2, 1.6, 0.4, 1);
}
@keyframes pop {
  from { transform: scale(0); }
  to { transform: scale(1); }
}
.success {
  border-color: rgba(46, 230, 168, 0.4);
}
.success h1 {
  color: var(--success);
}
.failed {
  border-color: rgba(253, 93, 143, 0.4);
}
.retry {
  margin-top: 14px;
  padding: 13px 30px;
  border: 1px solid var(--border-bright);
  border-radius: var(--radius-md);
  cursor: pointer;
  background: var(--glass-strong);
  color: var(--text);
  font-weight: 600;
  transition: background 0.15s;
}
.retry:hover {
  background: rgba(255, 255, 255, 0.18);
}
</style>
