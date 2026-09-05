<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api, type PublicProfile, type TopDonator } from '../api/auth'
import type { SoundKind } from '../api/auth'

const route = useRoute()
const router = useRouter()
const profile = ref<PublicProfile | null>(null)
const topDonators = ref<TopDonator[]>([])
const name = ref('')
const amount = ref<number | null>(null)
const message = ref('')
const sound = ref<SoundKind>('chime')
const submitting = ref(false)
const error = ref('')
const toast = ref(false)

// ?u=username → โดเนตให้สตรีมเมอร์คนนั้น (ลิงก์จาก Dashboard); ไม่ส่ง = streamer เดิมของระบบ
const targetUsername = (route.query.u as string) || undefined

onMounted(async () => {
  if (targetUsername) {
    try {
      profile.value = await api.publicProfile(targetUsername)
      if (profile.value.show_leaderboard) {
        topDonators.value = (await api.topDonators(targetUsername)).slice(0, 5)
      }
    } catch {
      error.value = 'ไม่พบสตรีมเมอร์นี้ — ตรวจลิงก์อีกครั้ง'
    }
  }
})

const presets = [20, 50, 100, 500]
const sounds: { value: SoundKind; label: string }[] = [
  { value: 'chime', label: '🔔 กระดิ่ง' },
  { value: 'coin', label: '🪙 เหรียญ' },
  { value: 'fanfare', label: '🎺 โห่ร้อง' },
  { value: 'tts', label: '🗣️ อ่านข้อความ (TTS)' },
]

async function submit() {
  error.value = ''
  if (!name.value.trim()) return (error.value = 'กรุณากรอกชื่อก่อนนะ')
  if (!amount.value || amount.value < 1) return (error.value = 'กรุณากรอกจำนวนเงินอย่างน้อย 1 บาท')

  submitting.value = true
  try {
    const res = await api.createDonation({
      name: name.value,
      amount: amount.value,
      message: message.value,
      sound: sound.value,
      username: targetUsername,
    })
    toast.value = true
    router.push({ path: `/pay/${res.id}`, query: { qr: res.qr_url, pay: res.pay_url } })
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'เกิดข้อผิดพลาด'
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="page">
    <div class="card">
      <div class="avatar">🎮</div>
      <h1>{{ profile ? profile.display_name : 'Donate Me' }}</h1>
      <p class="subtitle">
        {{ profile ? `สนับสนุน @${profile.username} 💜` : 'สนับสนุน Streamer เจ้าของช่อง เพื่อไฟล์ต่ออีกนานแล่น ๆ 💜' }}
      </p>

      <label>ชื่อ / ชื่อเล่น</label>
      <input v-model="name" placeholder="เช่น หมูทอดกรอบ" maxlength="50" />

      <label>จำนวนเงิน (บาท)</label>
      <input v-model.number="amount" type="number" min="1" max="100000" placeholder="เช่น 100" />
      <div class="amounts">
        <button
          v-for="p in presets"
          :key="p"
          class="amount-btn"
          :class="{ active: amount === p }"
          @click="amount = p"
        >
          {{ p }}
        </button>
      </div>

      <label>ข้อความถึงสตรีมเมอร์ (ไม่บังคับ)</label>
      <textarea v-model="message" rows="3" placeholder="สู้ ๆ นะ เก่งมากกก" maxlength="200" />

      <label>เสียงแจ้งเตือน</label>
      <select v-model="sound">
        <option v-for="s in sounds" :key="s.value" :value="s.value">{{ s.label }}</option>
      </select>

      <p v-if="error" class="error">{{ error }}</p>

      <button class="submit-btn" :disabled="submitting" @click="submit">
        {{ submitting ? 'กำลังสร้าง QR...' : '❤️ โดเนตเลย!' }}
      </button>

      <!-- 🏆 Top Donators -->
      <div v-if="topDonators.length" class="leaderboard">
        <h3>🏆 Top Donators</h3>
        <ol>
          <li v-for="(t, i) in topDonators" :key="t.donor_name">
            <span class="rank">{{ ['🥇', '🥈', '🥉'][i] ?? `#${i + 1}` }}</span>
            <span class="lb-name">{{ t.donor_name }}</span>
            <span class="lb-total">฿{{ t.total.toLocaleString() }}</span>
          </li>
        </ol>
      </div>

      <a class="obs-link" href="/overlay.html" target="_blank">
        🔊 เปิดหน้า Overlay สำหรับ OBS
      </a>
    </div>

    <div class="toast" :class="{ show: toast }">สร้างรายการโดเนตแล้ว ไปที่หน้าชำระเงิน...</div>
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
}
.card {
  width: 100%;
  max-width: 480px;
  background: rgba(255, 255, 255, 0.07);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 20px;
  padding: 36px 32px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  color: #fff;
}
.avatar {
  width: 90px;
  height: 90px;
  border-radius: 50%;
  background: linear-gradient(135deg, #ff6ec7, #7873f5);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 42px;
  margin: 0 auto 16px;
  box-shadow: 0 0 30px rgba(255, 110, 199, 0.5);
}
h1 {
  text-align: center;
  font-size: 26px;
  margin-bottom: 4px;
}
.subtitle {
  text-align: center;
  opacity: 0.7;
  margin-bottom: 28px;
  font-size: 14px;
}
label {
  display: block;
  font-size: 13px;
  opacity: 0.8;
  margin: 16px 0 6px;
}
input,
select,
textarea {
  width: 100%;
  padding: 12px 14px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  background: rgba(0, 0, 0, 0.25);
  color: #fff;
  font-size: 15px;
  outline: none;
  transition: border 0.2s;
}
input:focus,
textarea:focus,
select:focus {
  border-color: #ff6ec7;
}
select option {
  color: #111;
}
.amounts {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  margin-top: 8px;
}
.amount-btn {
  padding: 10px 0;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  background: rgba(0, 0, 0, 0.25);
  color: #fff;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.15s;
}
.amount-btn:hover {
  border-color: #ff6ec7;
}
.amount-btn.active {
  background: linear-gradient(135deg, #ff6ec7, #7873f5);
  border-color: transparent;
  box-shadow: 0 0 15px rgba(255, 110, 199, 0.5);
}
.error {
  color: #fda4af;
  font-size: 13px;
  margin-top: 12px;
  text-align: center;
}
.submit-btn {
  width: 100%;
  margin-top: 24px;
  padding: 15px;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  background: linear-gradient(135deg, #ff6ec7, #7873f5);
  color: #fff;
  font-size: 17px;
  font-weight: 700;
  transition:
    transform 0.15s,
    box-shadow 0.15s;
}
.submit-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 10px 25px rgba(255, 110, 199, 0.4);
}
.submit-btn:disabled {
  opacity: 0.6;
}
.obs-link {
  display: block;
  text-align: center;
  margin-top: 16px;
  font-size: 13px;
  color: #a5b4fc;
  text-decoration: none;
}
.obs-link:hover {
  text-decoration: underline;
}
.leaderboard {
  margin-top: 22px;
  background: rgba(0, 0, 0, 0.25);
  border-radius: 14px;
  padding: 16px 18px;
}
.leaderboard h3 {
  font-size: 14px;
  margin-bottom: 10px;
  opacity: 0.85;
}
.leaderboard ol {
  list-style: none;
  display: grid;
  gap: 6px;
}
.leaderboard li {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}
.lb-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.lb-total {
  color: #ffe066;
  font-weight: 700;
}
.toast {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%) translateY(-80px);
  background: #7873f5;
  color: #fff;
  padding: 12px 24px;
  border-radius: 12px;
  font-weight: 600;
  transition: transform 0.3s;
  z-index: 99;
}
.toast.show {
  transform: translateX(-50%) translateY(0);
}
</style>
