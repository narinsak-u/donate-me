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
const error = ref('')
let timer: ReturnType<typeof setInterval> | undefined

// โชว์ QR ได้ 15 นาที
const secondsLeft = ref(15 * 60)

onMounted(() => {
  timer = setInterval(async () => {
    secondsLeft.value = Math.max(0, secondsLeft.value - 1)
    try {
      const res = await api.getDonation(props.id)
      status.value = res.status
      if (res.status !== 'pending') {
        clearInterval(timer)
        // คงไว้ให้ดูผล 8 วิ แล้วกลับหน้าแรก
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
    <!-- กำลังรอชำระเงิน -->
    <div v-if="status === null || status === 'pending'" class="card">
      <h1>สแกนจ่ายผ่าน PromptPay</h1>
      <p class="subtitle">เปิดแอปธนาคาร → สแกน QR → ใส่จำนวนเงินตามที่กรอกไว้</p>

      <img v-if="qrUrl" :src="qrUrl" alt="PromptPay QR" class="qr" />
      <p class="countdown">QR หมดอายุใน {{ countdown }}</p>

      <a v-if="payUrl" :href="payUrl" target="_blank" class="mock-btn">
        🧪 จำลองการชำระเงิน (เปิดหน้า Mock Bank)
      </a>
      <p class="hint">ระบบ mockup — ยังไม่มีการตัดเงินจริง กดปุ่มด้านบนเพื่อจำลองว่าจ่ายแล้ว</p>
    </div>

    <!-- จ่ายสำเร็จ -->
    <div v-else-if="status === 'paid'" class="card success">
      <div class="big">🎉</div>
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

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.page {
  min-height: 100vh;
  background: linear-gradient(135deg, #1a1040 0%, #2d1b69 50%, #0f3460 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  color: #fff;
}
.card {
  width: 100%;
  max-width: 420px;
  text-align: center;
  background: rgba(255, 255, 255, 0.07);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 20px;
  padding: 36px 32px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
}
h1 {
  font-size: 24px;
  margin-bottom: 6px;
}
.subtitle {
  opacity: 0.75;
  font-size: 14px;
  margin-bottom: 24px;
}
.qr {
  width: 240px;
  height: 240px;
  background: #fff;
  border-radius: 16px;
  padding: 12px;
}
.countdown {
  margin-top: 16px;
  font-size: 14px;
  opacity: 0.8;
  font-variant-numeric: tabular-nums;
}
.mock-btn {
  display: block;
  margin-top: 20px;
  padding: 13px;
  border-radius: 12px;
  background: linear-gradient(135deg, #ff6ec7, #7873f5);
  color: #fff;
  font-weight: 700;
  text-decoration: none;
  font-size: 15px;
}
.mock-btn:hover {
  opacity: 0.92;
}
.hint {
  margin-top: 12px;
  font-size: 12px;
  opacity: 0.55;
}
.big {
  font-size: 64px;
}
.success {
  border-color: rgba(34, 197, 94, 0.5);
}
.failed {
  border-color: rgba(244, 63, 94, 0.5);
}
.retry {
  margin-top: 12px;
  padding: 13px 28px;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
  font-weight: 600;
}
.error {
  color: #fda4af;
  margin-top: 16px;
}
</style>
