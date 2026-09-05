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
const anonymous = ref(false)
const submitting = ref(false)
const error = ref('')

const targetUsername = (route.query.u as string) || undefined

// theme toggle (dark = Minimal & Friendly, light = Clean Light Mode)
const theme = ref(localStorage.getItem('donateme_theme') ?? 'dark')
function toggleTheme() {
  theme.value = theme.value === 'dark' ? 'light' : 'dark'
  localStorage.setItem('donateme_theme', theme.value)
  document.documentElement.setAttribute('data-theme', theme.value)
}
onMounted(() => {
  document.documentElement.setAttribute('data-theme', theme.value)
  if (targetUsername) {
    api.publicProfile(targetUsername)
      .then(async (p) => {
        profile.value = p
        if (p.show_leaderboard) topDonators.value = (await api.topDonators(targetUsername)).slice(0, 5)
      })
      .catch(() => (error.value = 'ไม่พบสตรีมเมอร์นี้ — ตรวจลิงก์อีกครั้ง'))
  }
})

const presets = [
  { v: 20, label: 'กาแฟวาร์ป', emoji: '☕' },
  { v: 50, label: 'เชลบี้', emoji: '⚡' },
  { v: 100, label: 'อัคคีไฟ', emoji: '🔥' },
  { v: 500, label: 'ชุดปรับเกียร์', emoji: '🔧' },
]
const sounds: { value: SoundKind; label: string; color: string }[] = [
  { value: 'chime', label: 'กระดิ่ง', color: '#f43f5e' },
  { value: 'coin', label: 'เหรียญ', color: '#fbbf24' },
  { value: 'fanfare', label: 'แตรวง', color: '#10b981' },
  { value: 'tts', label: 'เสียงพูด', color: '#3b82f6' },
]

async function submit() {
  error.value = ''
  if (!name.value.trim()) return (error.value = 'กรุณากรอกชื่อก่อนนะ')
  if (!amount.value || amount.value < 1) return (error.value = 'กรุณากรอกจำนวนเงินอย่างน้อย 1 บาท')

  submitting.value = true
  try {
    const res = await api.createDonation({
      name: anonymous.value ? 'ไม่ระบุชื่อ' : name.value,
      amount: amount.value,
      message: message.value,
      sound: sound.value,
      username: targetUsername,
    })
    router.push({ path: `/pay/${res.id}`, query: { qr: res.qr_url, pay: res.pay_url, amount: String(amount.value) } })
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'เกิดข้อผิดพลาด'
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <div class="page">
    <!-- ===== Header ===== -->
    <header class="topbar">
      <div class="logo"><span class="logo-icon">❤️</span> <b>Donate Me</b><span class="logo-heart">❤️</span></div>
      <nav class="topnav">
        <a href="#" class="active">หน้าแรก</a>
        <a href="#">คู่มือสตรีมเมอร์</a>
        <a href="#" class="pill">จุดรับโดเนต</a>
      </nav>
      <div class="topbar-right">
        <button class="theme-toggle" title="สลับธีม" @click="toggleTheme">{{ theme === 'dark' ? '☀️' : '🌙' }}</button>
        <a href="/#/dashboard" class="join-btn">เข้าร่วม</a>
        <div class="me-avatar">ME</div>
      </div>
    </header>

    <main class="wrap">
      <!-- ===== Streamer hero ===== -->
      <section class="hero">
        <div class="hero-avatar"><span>{{ profile?.display_name?.[0] ?? '🎮' }}</span></div>
        <div class="hero-info">
          <div class="hero-name-row">
            <h1>{{ profile?.display_name ?? 'Donate Me' }}</h1>
            <span v-if="profile" class="partner-badge">PARTNER</span>
            <span v-if="profile" class="follower-pill">🔴 ผู้ติดตามล่าสุด · 1,420 คน</span>
          </div>
          <p class="hero-bio">
            {{ profile ? `ยินดีต้อนรับสู่หน้าโดเนตของ @${profile.username} — ทุกการสนับสนุนช่วยให้ไฟล์ต่อได้นานและสนุกยิ่งขึ้น 💜` : 'สนับสนุนสตรีมเมอร์คนโปรด ทุกการโดเนตช่วยให้ไฟล์ต่อได้ต่อเนื่องและสนุกยิ่งขึ้น 💜' }}
          </p>
          <div v-if="profile && profile.goal_amount > 0" class="goal-box">
            <div class="goal-head">
              <span class="goal-title">🎙️ ปั้นทุนไมค์ใหม่ Shure SM7B</span>
              <span class="goal-pct">{{ Math.min(100, Math.round((topDonators.reduce((s, t) => s + t.total, 0) / profile.goal_amount) * 100)) || 78 }}%</span>
            </div>
            <div class="goal-bar"><div class="goal-fill" /></div>
            <div class="goal-foot">
              <span>฿{{ topDonators.reduce((s, t) => s + t.total, 0).toLocaleString() }} สะสมแล้ว</span>
              <span>เป้าหมาย ฿{{ profile.goal_amount.toLocaleString() }}</span>
            </div>
          </div>
        </div>
      </section>

      <!-- ===== Form + QR ===== -->
      <section class="grid-2">
        <div class="card form-card">
          <div class="card-head">
            <h2>❤️ ส่งกำลังใจสนับสนุน</h2>
            <span class="badge badge-pink">Alert เด้งสด ๆ ทันที</span>
          </div>

          <label>เลือกวงเงินสนับสนุน (บาท)</label>
          <div class="presets">
            <button
              v-for="p in presets"
              :key="p.v"
              class="preset"
              :class="{ active: amount === p.v }"
              @click="amount = p.v"
            >
              <span class="preset-label">{{ p.label }} <i>{{ p.emoji }}</i></span>
              <b>฿{{ p.v }}</b>
            </button>
          </div>
          <div class="amount-input">
            <span class="thb">฿</span>
            <input v-model.number="amount" type="number" min="1" max="100000" placeholder="100" />
            <span class="suffix">THB</span>
          </div>

          <div class="name-row">
            <div class="name-field">
              <label>ชื่อผู้ส่งกำลังใจ</label>
              <input v-model="name" class="input" placeholder="NongSomZa_77" maxlength="50" />
            </div>
            <label class="anon">
              <input v-model="anonymous" type="checkbox" />
              <span>ไม่แสดงชื่อ</span>
            </label>
          </div>

          <div class="msg-head">
            <label>ข้อความถึงสตรีมเมอร์ (Alert Popup)</label>
            <span class="counter">{{ message.length }}/150</span>
          </div>
          <textarea v-model="message" class="input" rows="3" placeholder="สู้ ๆ นะพี่พีช วันนี้เล่น Valorant ด้วยวาเนนะ สู้สู้ปุย! 💗" maxlength="150" />

          <div class="sound-box">
            <div class="sound-head">
              <span>🎵 เลือกเสียงประกอบด้วย AI</span>
              <span class="badge badge-green">เลือกความยาว</span>
            </div>
            <div class="sound-chips">
              <button
                v-for="s in sounds"
                :key="s.value"
                class="sound-chip"
                :class="{ active: sound === s.value }"
                @click="sound = s.value"
              >
                <i :style="{ background: s.color }" /> {{ s.label }}
              </button>
            </div>
            <label class="tts-row">
              <input v-model="sound" type="radio" :value="'tts'" :checked="sound === 'tts'" @click="sound = sound === 'tts' ? 'chime' : 'tts'" />
              <span>🗣️ ให้ระบบอ่านข้อความไทยออกเสียง (TTS)</span>
              <em class="tts-speed">เร่งความเร็ว 1.0x</em>
            </label>
          </div>

          <div class="preview-strip">
            <div class="preview-label">ตัวอย่างการแจ้งเตือนสด (Preview)</div>
            <div class="preview-body">
              <b>{{ anonymous ? 'ไม่ระบุชื่อ' : name || 'ชื่อของคุณ' }}</b> สนับสนุน <em>฿{{ (amount || 0).toLocaleString() }}</em> : "{{ message || 'ข้อความของคุณ' }}"
            </div>
          </div>

          <p v-if="error" class="error">{{ error }}</p>

          <button class="btn-primary cta" :disabled="submitting" @click="submit">
            {{ submitting ? 'กำลังสร้าง QR...' : '⚡ สร้างคิวอาร์พร้อมเพย์ (PromptPay)' }}
          </button>
        </div>

        <div class="side-col">
          <div class="card qr-card">
            <div class="card-head">
              <span class="badge badge-green">● เชื่อมต่อระบบสำเร็จ</span>
              <span class="qr-tag">Thai QR Payment</span>
            </div>
            <div class="qr-mini">
              <div class="qr-mini-head">PromptPay พร้อมเพย์</div>
              <div class="qr-mini-body">
                <img src="/favicon.svg" alt="QR" />
                <p>สแกนผ่านแอปธนาคาร<br />(ปรากฏหลังกดปุ่มด้านซ้าย)</p>
              </div>
              <div class="qr-mini-amount">฿{{ (amount || 0).toLocaleString() }}.00</div>
            </div>
            <div class="qr-note-box">
              <b>⏳ กรุณาระบบใน 15 นาที</b>
              <p>เมื่อระบบตรวจสอบรายการ สถานะจะเปลี่ยนเป็นสำเร็จ Alert จะเด้งในฉากสตรีมภายใน 1-3 วินาที</p>
            </div>
            <div class="secure-note">
              🔒 ระบบนี้ปลอดภัยด้วยกระบวนการยืนยันระดับธนาคาร — เป็นเพียงระบบจำลอง (Mock) ยังไม่มีการตัดเงินจริง 100%
            </div>
          </div>
        </div>
      </section>

      <!-- ===== Leaderboard + Live feed ===== -->
      <section class="grid-2">
        <div class="card">
          <div class="card-head">
            <h2>🏆 ผู้สนับสนุนสูงสุดประจำเดือน</h2>
            <span class="muted">Top 5</span>
          </div>
          <ol v-if="topDonators.length" class="board">
            <li v-for="(t, i) in topDonators" :key="t.donor_name">
              <span class="rank" :class="`rank-${i + 1}`">{{ i + 1 }}</span>
              <div class="board-info">
                <b>{{ t.donor_name }}</b>
                <span>{{ t.count }} ครั้ง</span>
              </div>
              <span class="board-amount">฿{{ t.total.toLocaleString() }}</span>
            </li>
          </ol>
          <p v-else class="muted empty">ยังไม่มีข้อมูล — เป็นคนแรกที่สนับสนุนสิ!</p>
        </div>

        <div class="card">
          <div class="card-head">
            <h2>⚡ กำลังส่งกำลังใจ สด ๆ</h2>
            <span class="badge badge-green">● ระบบทำงาน</span>
          </div>
          <div class="feed">
            <div class="feed-item">
              <span class="feed-icon">❤️</span>
              <div><b>Gunz_lnwZa</b> <em class="feed-amt">฿300.00</em> <span class="muted small">· 1 นาทีที่แล้ว</span>
                <p>"พี่แนนสตรีมมิ่งเพราะสุดในจักรวาลเลยครับ ราศีนี้ครับทีมเดียวกัน"</p>
              </div>
            </div>
            <div class="feed-item">
              <span class="feed-icon">⭐</span>
              <div><b>คุณปริสนธ์สปอนเซอร์</b> <em class="feed-amt">฿100.00</em> <span class="muted small">· 4 นาทีที่แล้ว</span>
                <p>"เว็บทำได้ดีมากครับ โหนดแมนย้อนหลัง ขอแสดงเป็นกำลังใจและกำลัง 💗"</p>
              </div>
            </div>
            <div class="feed-item">
              <span class="feed-icon">⚡</span>
              <div><b>Pluem_Ch</b> <em class="feed-amt">฿50.00</em> <span class="muted small">· 12 นาทีที่แล้ว</span>
                <p>"กราฟวิ่งไหว โหนดกานบนอกอุ่นมาก 5555"</p>
              </div>
            </div>
          </div>
          <a class="feed-more" href="/#/dashboard">ดูรายการทั้งหมดสนุกต่ออีก →</a>
        </div>
      </section>

      <footer class="footer">
        <span><b>Donate Me</b> ❤️ แพลตฟอร์มสนับสนุนสตรีมเมอร์คนไทยโดยเฉพาะ</span>
        <span class="muted">© 2026 Donate Me. ออกแบบด้วยรัรัก ให้ทุกคนกล้าส่งเสียง</span>
      </footer>
    </main>
  </div>
</template>

<style scoped>
.page { min-height: 100vh; background: var(--bg); }
.topbar {
  position: sticky; top: 0; z-index: 10;
  display: flex; align-items: center; gap: 28px;
  padding: 12px 32px;
  background: color-mix(in srgb, var(--bg) 85%, transparent);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--border);
}
.logo { display: flex; align-items: center; gap: 7px; font-family: var(--font-head); font-size: 17px; }
.logo-icon {
  display: inline-flex; align-items: center; justify-content: center;
  width: 34px; height: 34px; border-radius: 11px; font-size: 16px;
  background: var(--primary-soft);
}
.logo-heart { font-size: 11px; }
.topnav { display: flex; gap: 6px; flex: 1; }
.topnav a {
  padding: 8px 15px; border-radius: 999px; font-size: 13.5px; font-weight: 600;
  color: var(--text-dim); text-decoration: none;
}
.topnav a.active { color: var(--primary); background: var(--primary-soft); }
.topnav a.pill { border: 1px solid var(--border-bright); }
.topbar-right { display: flex; align-items: center; gap: 12px; }
.join-btn {
  padding: 9px 20px; border-radius: 999px; border: 1px solid var(--border-bright);
  color: var(--text); font-size: 13.5px; font-weight: 700; text-decoration: none;
}
.me-avatar {
  width: 36px; height: 36px; border-radius: 50%;
  background: linear-gradient(135deg, var(--primary), #fb7185);
  display: flex; align-items: center; justify-content: center;
  font-size: 11px; font-weight: 800; color: #fff;
}
.wrap { max-width: 1180px; margin: 0 auto; padding: 24px 28px 40px; display: grid; gap: 22px; }

/* hero */
.hero {
  display: flex; gap: 20px; align-items: flex-start;
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius-xl); padding: 24px; box-shadow: var(--shadow-card);
}
.hero-avatar {
  flex-shrink: 0;
  width: 92px; height: 92px; border-radius: 22px; overflow: hidden;
  background: linear-gradient(135deg, var(--primary), #a855f7);
  display: flex; align-items: center; justify-content: center;
  font-size: 40px; font-family: var(--font-head); font-weight: 800; color: #fff;
}
.hero-info { flex: 1; }
.hero-name-row { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
h1 { font-size: 27px; font-weight: 800; }
.partner-badge {
  padding: 3px 10px; border-radius: 7px; font-size: 10.5px; font-weight: 800; letter-spacing: 0.5px;
  background: var(--emerald-soft); color: var(--emerald);
}
.follower-pill {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 3px 11px; border-radius: 999px; font-size: 11.5px; color: var(--text-dim);
  border: 1px solid var(--border);
}
.hero-bio { margin-top: 7px; color: var(--text-dim); font-size: 14px; line-height: 1.65; }
.goal-box {
  margin-top: 16px; padding: 14px 16px; border-radius: var(--radius-lg);
  background: var(--bg-card-2); border: 1px solid var(--border);
}
.goal-head { display: flex; justify-content: space-between; font-size: 13.5px; font-weight: 700; }
.goal-pct { color: var(--emerald); }
.goal-bar { height: 8px; border-radius: 999px; background: var(--border); margin: 10px 0 8px; overflow: hidden; }
.goal-fill { height: 100%; width: 78%; border-radius: 999px; background: linear-gradient(90deg, var(--primary), var(--emerald)); }
.goal-foot { display: flex; justify-content: space-between; font-size: 12px; color: var(--text-faint); }

/* grid */
.grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 22px; align-items: start; }
.card {
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius-xl); padding: 22px; box-shadow: var(--shadow-card);
}
.card-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; }
.card-head h2 { font-size: 16.5px; font-weight: 700; }
.muted { color: var(--text-faint); font-size: 12.5px; }
.small { font-size: 11.5px; }

/* form */
label { display: block; font-size: 13px; font-weight: 600; color: var(--text-dim); margin: 14px 0 7px; }
.presets { display: grid; grid-template-columns: repeat(4, 1fr); gap: 9px; }
.preset {
  padding: 11px 8px; border-radius: var(--radius-md); text-align: left;
  border: 1px solid var(--border); background: var(--bg-input); color: var(--text);
  cursor: pointer; transition: all 0.15s; font-family: var(--font-body);
}
.preset-label { display: block; font-size: 10.5px; color: var(--text-faint); margin-bottom: 3px; }
.preset-label i { font-style: normal; }
.preset b { font-family: var(--font-head); font-size: 16px; }
.preset:hover { border-color: var(--border-bright); }
.preset.active {
  background: linear-gradient(135deg, #fb7185, var(--primary));
  border-color: transparent; box-shadow: 0 6px 20px rgba(244, 63, 94, 0.4);
}
.preset.active .preset-label { color: rgba(255, 255, 255, 0.8); }
.amount-input {
  display: flex; align-items: center; gap: 8px;
  margin-top: 10px; padding: 0 14px;
  border: 1px solid var(--border); border-radius: var(--radius-md); background: var(--bg-input);
}
.amount-input:focus-within { border-color: var(--primary); box-shadow: 0 0 0 3px var(--primary-soft); }
.amount-input .thb { color: var(--text-faint); }
.amount-input input {
  flex: 1; border: none; background: transparent; padding: 12px 0;
  color: var(--text); font-size: 15px; outline: none;
}
.amount-input .suffix { font-size: 12px; color: var(--text-faint); font-weight: 700; }
.name-row { display: flex; align-items: flex-end; gap: 14px; }
.name-field { flex: 1; }
.anon {
  display: flex; align-items: center; gap: 7px; margin-bottom: 13px; cursor: pointer;
  font-size: 12.5px; color: var(--text-dim); white-space: nowrap;
}
.anon input { width: auto; accent-color: var(--primary); }
.msg-head { display: flex; justify-content: space-between; align-items: center; }
.counter { font-size: 11px; color: var(--text-faint); }
textarea { resize: vertical; }

.sound-box {
  margin-top: 16px; padding: 14px; border-radius: var(--radius-lg);
  background: var(--bg-card-2); border: 1px solid var(--border);
}
.sound-head { display: flex; justify-content: space-between; font-size: 13px; font-weight: 700; }
.sound-chips { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin-top: 11px; }
.sound-chip {
  display: flex; align-items: center; gap: 7px; padding: 9px 10px;
  border-radius: var(--radius-sm); border: 1px solid var(--border);
  background: var(--bg-input); color: var(--text-dim);
  font-size: 12.5px; font-family: var(--font-body); cursor: pointer; transition: all 0.15s;
}
.sound-chip i { width: 8px; height: 8px; border-radius: 50%; }
.sound-chip:hover { border-color: var(--border-bright); color: var(--text); }
.sound-chip.active { border-color: var(--primary); color: var(--text); background: var(--primary-soft); }
.tts-row {
  display: flex; align-items: center; gap: 9px; margin: 12px 0 0; cursor: pointer;
  font-size: 12.5px; color: var(--text-dim); font-weight: 500;
}
.tts-row input { width: auto; accent-color: var(--primary); }
.tts-speed { margin-left: auto; font-style: normal; font-size: 11px; color: var(--text-faint); }

.preview-strip {
  margin-top: 16px; padding: 13px 15px; border-radius: var(--radius-lg);
  background: var(--bg-card-2); border: 1px solid var(--border);
}
.preview-label { font-size: 11px; color: var(--text-faint); margin-bottom: 7px; }
.preview-body { font-size: 13px; color: var(--text-dim); line-height: 1.6; }
.preview-body b { color: var(--primary); }
.preview-body em { font-style: normal; color: var(--gold); font-weight: 700; }
.error { color: var(--primary); font-size: 13px; margin-top: 12px; text-align: center; }
.cta { width: 100%; margin-top: 20px; padding: 16px; font-size: 16.5px; }

/* QR side */
.qr-tag { font-size: 12px; color: var(--text-faint); font-weight: 600; }
.qr-mini {
  border-radius: var(--radius-lg); overflow: hidden; border: 1px solid var(--border);
  background: #fff; color: #0f172a; max-width: 320px; margin: 6px auto 0;
}
.qr-mini-head {
  background: #1e3a8a; color: #fff; text-align: center;
  font-size: 13px; font-weight: 700; padding: 9px;
}
.qr-mini-body { text-align: center; padding: 18px 20px 10px; }
.qr-mini-body img { width: 110px; height: 110px; opacity: 0.35; }
.qr-mini-body p { font-size: 11.5px; color: #64748b; margin-top: 8px; }
.qr-mini-amount {
  text-align: center; font-family: var(--font-head); font-size: 21px; font-weight: 800;
  padding: 8px 0 16px;
}
.qr-note-box {
  margin-top: 14px; padding: 13px 15px; border-radius: var(--radius-md);
  background: var(--bg-card-2); border: 1px solid var(--border); text-align: center;
}
.qr-note-box b { font-size: 13px; color: var(--gold); }
.qr-note-box p { font-size: 12px; color: var(--text-dim); margin-top: 5px; line-height: 1.6; }
.secure-note {
  margin-top: 12px; font-size: 11.5px; color: var(--text-faint); line-height: 1.6;
  padding: 10px 12px; border-radius: var(--radius-sm); background: var(--emerald-soft);
}

/* leaderboard + feed */
.board { list-style: none; display: grid; gap: 8px; }
.board li {
  display: flex; align-items: center; gap: 12px; padding: 10px 12px;
  border-radius: var(--radius-md); border: 1px solid var(--border); background: var(--bg-input);
}
.rank {
  width: 28px; height: 28px; border-radius: 50%; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  font-size: 12.5px; font-weight: 800; background: var(--bg-card-2); color: var(--text-dim);
}
.rank-1 { background: var(--gold); color: #422006; }
.rank-2 { background: #cbd5e1; color: #334155; }
.rank-3 { background: #f59e0b; color: #431407; }
.board-info { flex: 1; display: flex; flex-direction: column; }
.board-info span { font-size: 11px; color: var(--text-faint); }
.board-amount { font-family: var(--font-head); font-weight: 700; color: var(--gold); }
.empty { padding: 20px; text-align: center; }

.feed { display: grid; gap: 10px; }
.feed-item {
  display: flex; gap: 11px; padding: 11px 13px;
  border-radius: var(--radius-md); border: 1px solid var(--border); background: var(--bg-input);
}
.feed-icon {
  width: 32px; height: 32px; border-radius: 10px; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center; font-size: 15px;
  background: var(--bg-card-2);
}
.feed-item b { font-size: 13px; }
.feed-amt { font-style: normal; color: var(--emerald); font-weight: 700; font-size: 12.5px; }
.feed-item p { font-size: 12px; color: var(--text-dim); margin-top: 3px; line-height: 1.55; }
.feed-more {
  display: block; text-align: center; margin-top: 14px;
  color: var(--primary); font-size: 13px; font-weight: 700; text-decoration: none;
}

.footer {
  display: flex; justify-content: space-between; gap: 12px; flex-wrap: wrap;
  padding-top: 18px; border-top: 1px solid var(--border); font-size: 12.5px; color: var(--text-dim);
}

@media (max-width: 900px) {
  .grid-2 { grid-template-columns: 1fr; }
  .hero { flex-direction: column; }
}
</style>
