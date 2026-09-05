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
    <div class="aurora" />
    <div class="stars" />

    <div class="card">
      <div class="avatar">
        <span>{{ profile?.display_name?.[0] ?? '🎮' }}</span>
      </div>
      <h1>{{ profile ? profile.display_name : 'Donate Me' }}</h1>
      <p class="subtitle">
        <template v-if="profile">สนับสนุน <strong>@{{ profile.username }}</strong> และช่วยให้ไฟล์ต่อไปได้นาน ๆ 💜</template>
        <template v-else>สนับสนุนสตรีมเมอร์คนโปรด เพื่อไฟล์ต่ออีกนานแล่น ๆ 💜</template>
      </p>

      <div class="field">
        <label>ชื่อ / ชื่อเล่น</label>
        <input v-model="name" placeholder="เช่น หมูทอดกรอบ" maxlength="50" />
      </div>

      <div class="field">
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
            ฿{{ p }}
          </button>
        </div>
      </div>

      <div class="field">
        <label>ข้อความถึงสตรีมเมอร์ <span class="opt">(ไม่บังคับ)</span></label>
        <textarea v-model="message" rows="3" placeholder="สู้ ๆ นะ เก่งมากกก" maxlength="200" />
      </div>

      <div class="field">
        <label>เสียงแจ้งเตือน</label>
        <div class="sounds">
          <button
            v-for="s in sounds"
            :key="s.value"
            class="sound-btn"
            :class="{ active: sound === s.value }"
            @click="sound = s.value"
          >
            {{ s.label }}
          </button>
        </div>
      </div>

      <p v-if="error" class="error">{{ error }}</p>

      <button class="submit-btn" :disabled="submitting" @click="submit">
        <span class="heart">❤️</span>
        {{ submitting ? 'กำลังสร้าง QR...' : 'โดเนตเลย!' }}
      </button>

      <a class="obs-link" href="/overlay.html" target="_blank">🔊 เปิดหน้า Overlay สำหรับ OBS</a>

      <!-- 🏆 Top Donators -->
      <div v-if="topDonators.length" class="leaderboard">
        <h3>🏆 Top Donators</h3>
        <ol>
          <li v-for="(t, i) in topDonators" :key="t.donor_name">
            <span class="rank">{{ ['🥇', '🥈', '🥉'][i] ?? `#${i + 1}` }}</span>
            <span class="lb-name">{{ t.donor_name }}</span>
            <span class="lb-count">{{ t.count }} ครั้ง</span>
            <span class="lb-total">฿{{ t.total.toLocaleString() }}</span>
          </li>
        </ol>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 28px 20px;
  position: relative;
  background: linear-gradient(160deg, var(--bg-0) 0%, var(--bg-1) 45%, #0f3460 100%);
}
.card {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 500px;
  background: var(--glass);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 40px 36px 32px;
  box-shadow: var(--shadow-card);
}
.avatar {
  width: 96px;
  height: 96px;
  border-radius: 32px;
  background: linear-gradient(135deg, var(--accent-1), var(--accent-2));
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 40px;
  font-family: var(--font-head);
  font-weight: 700;
  margin: -68px auto 18px;
  border: 4px solid rgba(255, 255, 255, 0.25);
  box-shadow: var(--glow-pink);
}
h1 {
  text-align: center;
  font-size: 30px;
  font-weight: 700;
  background: linear-gradient(90deg, #fff, #e8dcff);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.subtitle {
  text-align: center;
  color: var(--text-dim);
  margin: 6px 0 30px;
  font-size: 14.5px;
}
.subtitle strong {
  color: var(--accent-1);
}
.field {
  margin-bottom: 18px;
}
label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-dim);
  margin-bottom: 7px;
  letter-spacing: 0.3px;
}
.opt {
  color: var(--text-faint);
  font-weight: 400;
}
input,
textarea {
  width: 100%;
  padding: 13px 16px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
  background: rgba(0, 0, 0, 0.3);
  color: var(--text);
  font-size: 15px;
  font-family: var(--font-body);
  outline: none;
  transition: border 0.2s, box-shadow 0.2s;
}
input:focus,
textarea:focus {
  border-color: var(--accent-1);
  box-shadow: 0 0 0 3px rgba(255, 110, 199, 0.18);
}
textarea {
  resize: vertical;
}
.amounts {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  margin-top: 10px;
}
.amount-btn {
  padding: 11px 0;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: rgba(0, 0, 0, 0.25);
  color: var(--text-dim);
  cursor: pointer;
  font-weight: 600;
  font-size: 14px;
  transition: all 0.15s;
}
.amount-btn:hover {
  border-color: var(--accent-1);
  color: var(--text);
}
.amount-btn.active {
  background: linear-gradient(135deg, var(--accent-1), var(--accent-2));
  border-color: transparent;
  color: #fff;
  box-shadow: 0 4px 18px rgba(255, 110, 199, 0.4);
}
.sounds {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.sound-btn {
  padding: 11px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: rgba(0, 0, 0, 0.25);
  color: var(--text-dim);
  cursor: pointer;
  font-size: 13px;
  font-family: var(--font-body);
  transition: all 0.15s;
}
.sound-btn:hover {
  border-color: var(--accent-2);
  color: var(--text);
}
.sound-btn.active {
  background: rgba(120, 115, 245, 0.25);
  border-color: var(--accent-2);
  color: var(--text);
  box-shadow: 0 0 0 3px rgba(120, 115, 245, 0.15);
}
.error {
  color: var(--danger);
  font-size: 13px;
  margin: 12px 0 0;
  text-align: center;
}
.submit-btn {
  width: 100%;
  margin-top: 24px;
  padding: 17px;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  background: linear-gradient(135deg, var(--accent-1), var(--accent-2));
  color: #fff;
  font-size: 18px;
  font-weight: 700;
  font-family: var(--font-head);
  letter-spacing: 0.5px;
  transition: transform 0.15s, box-shadow 0.2s;
  box-shadow: 0 8px 30px rgba(255, 110, 199, 0.35);
}
.submit-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 12px 40px rgba(255, 110, 199, 0.5);
}
.submit-btn:disabled {
  opacity: 0.6;
}
.heart {
  display: inline-block;
  margin-right: 8px;
  animation: heartbeat 1.4s ease-in-out infinite;
}
@keyframes heartbeat {
  0%, 100% { transform: scale(1); }
  12% { transform: scale(1.25); }
  24% { transform: scale(1); }
  36% { transform: scale(1.18); }
  48% { transform: scale(1); }
}
.obs-link {
  display: block;
  text-align: center;
  margin-top: 18px;
  font-size: 13px;
  color: #a5b4fc;
  text-decoration: none;
}
.obs-link:hover {
  text-decoration: underline;
}
.leaderboard {
  margin-top: 26px;
  background: rgba(0, 0, 0, 0.28);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 18px 20px;
}
.leaderboard h3 {
  font-size: 14px;
  margin-bottom: 12px;
  color: var(--gold);
}
.leaderboard ol {
  list-style: none;
  display: grid;
  gap: 8px;
}
.leaderboard li {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13.5px;
}
.rank {
  width: 26px;
  text-align: center;
}
.lb-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.lb-count {
  font-size: 11px;
  color: var(--text-faint);
}
.lb-total {
  color: var(--gold);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}
</style>
