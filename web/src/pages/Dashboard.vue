<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { api, getToken, getUser, type PublicUser, type Settings } from '../api/auth'
import { dashApi, type DonationItem, type Stats } from '../api/dashboard'
import DonationChart from '../components/DonationChart.vue'

const router = useRouter()
const user = ref<PublicUser | null>(getUser())
const tab = ref<'overview' | 'donations' | 'settings'>('overview')
const donateLink = ref('')
const copied = ref(false)

// overview
const stats = ref<Stats | null>(null)
// donations
const items = ref<DonationItem[]>([])
const search = ref('')
const page = ref(1)
const totalPages = ref(1)
// settings
const settings = ref<Settings | null>(null)
const soundFile = ref<File | null>(null)
const soundUrl = ref('')

onMounted(async () => {
  if (!getToken()) {
    router.push('/auth')
    return
  }
  if (user.value) donateLink.value = `${location.origin}/?u=${user.value.username}`
  await loadAll()
})

async function loadAll() {
  const [s, st] = await Promise.all([dashApi.stats(), api.getSettings()])
  stats.value = s
  settings.value = st
  await loadHistory()
}

async function loadHistory() {
  const res = await dashApi.history({ query: search.value, page: page.value })
  items.value = res.items
  totalPages.value = res.total_pages
}

function copyLink() {
  navigator.clipboard.writeText(donateLink.value)
  copied.value = true
  setTimeout(() => (copied.value = false), 2000)
}

async function toggleHidden(item: DonationItem) {
  await dashApi.setHidden(item.id, !item.hidden)
  item.hidden = !item.hidden
}

async function save() {
  if (!settings.value) return
  try {
    settings.value = await api.updateSettings(settings.value)
    alert('บันทึกแล้ว!')
  } catch (e) {
    alert(e instanceof Error ? e.message : 'บันทึกไม่สำเร็จ')
  }
}

async function uploadSound() {
  if (!soundFile.value) return
  try {
    const res = await dashApi.uploadSound(soundFile.value)
    soundUrl.value = res.url
    alert('อัปโหลดเสียงสำเร็จ!')
  } catch (e) {
    alert(e instanceof Error ? e.message : 'อัปโหลดไม่สำเร็จ')
  }
}

async function testAlert() {
  await api.testAlert()
}

function logout() {
  localStorage.clear()
  router.push('/auth')
}

function fmtDate(ms: number | null): string {
  if (!ms) return '-'
  return new Date(ms).toLocaleString('th-TH', { dateStyle: 'short', timeStyle: 'short' })
}
</script>

<template>
  <div class="page">
    <header>
      <span class="logo">💜 Donate Me</span>
      <span v-if="user" class="who">{{ user.display_name }} (@{{ user.username }})</span>
      <button class="ghost" @click="logout">ออกจากระบบ</button>
    </header>

    <nav>
      <button v-for="t in ['overview', 'donations', 'settings'] as const" :key="t" :class="{ active: tab === t }" @click="tab = t">
        {{ t === 'overview' ? 'ภาพรวม' : t === 'donations' ? 'ประวัติโดเนต' : 'ตั้งค่า' }}
      </button>
    </nav>

    <main>
      <!-- ===== ภาพรวม ===== -->
      <template v-if="tab === 'overview' && stats">
        <section class="card link-card">
          <h2>🔗 ลิงก์รับโดเนต</h2>
          <div class="link-row">
            <code>{{ donateLink }}</code>
            <button class="primary" @click="copyLink">{{ copied ? '✓ แล้ว' : 'คัดลอก' }}</button>
          </div>
        </section>

        <div class="stat-grid">
          <div class="card stat"><span class="num">฿{{ stats.total_today.toLocaleString() }}</span><span>วันนี้</span></div>
          <div class="card stat"><span class="num">฿{{ stats.total_month.toLocaleString() }}</span><span>เดือนนี้</span></div>
          <div class="card stat"><span class="num">฿{{ stats.total_all.toLocaleString() }}</span><span>รวมทั้งหมด</span></div>
          <div class="card stat"><span class="num">{{ stats.count_all }}</span><span>จำนวนครั้ง</span></div>
        </div>

        <section class="card">
          <h2>📈 ยอดโดเนต 30 วันล่าสุด</h2>
          <DonationChart :data="stats.series_30d" :goal="stats.goal_amount" />
        </section>
      </template>

      <!-- ===== ประวัติ ===== -->
      <template v-if="tab === 'donations'">
        <section class="card">
          <div class="toolbar">
            <input v-model="search" placeholder="ค้นหาชื่อ/ข้อความ..." @keyup.enter="page = 1; loadHistory()" />
            <button class="ghost" @click="page = 1; loadHistory()">ค้นหา</button>
            <a href="/api/me/donations.csv" class="ghost-link">⬇ Export CSV</a>
          </div>
          <table>
            <thead>
              <tr><th>เวลา</th><th>ชื่อ</th><th>ยอด</th><th>ข้อความ</th><th></th></tr>
            </thead>
            <tbody>
              <tr v-for="it in items" :key="it.id">
                <td class="muted">{{ fmtDate(it.paid_at) }}</td>
                <td>{{ it.donor_name }}</td>
                <td class="amount">฿{{ it.amount.toLocaleString() }}</td>
                <td :class="{ hidden: it.hidden }">{{ it.hidden ? '(ซ่อนแล้ว)' : it.message || '-' }}</td>
                <td>
                  <button class="mini" @click="toggleHidden(it)">{{ it.hidden ? 'แสดง' : 'ซ่อน' }}</button>
                </td>
              </tr>
              <tr v-if="!items.length"><td colspan="5" class="muted center">ยังไม่มีประวัติโดเนต</td></tr>
            </tbody>
          </table>
          <div class="pager" v-if="totalPages > 1">
            <button class="mini" :disabled="page <= 1" @click="page--; loadHistory()">← ก่อนหน้า</button>
            <span>{{ page }} / {{ totalPages }}</span>
            <button class="mini" :disabled="page >= totalPages" @click="page++; loadHistory()">ถัดไป →</button>
          </div>
        </section>
      </template>

      <!-- ===== ตั้งค่า ===== -->
      <template v-if="tab === 'settings' && settings">
        <section class="card">
          <h2>🔔 ตั้งค่าแจ้งเตือน</h2>

          <label>ธีมสี</label>
          <select v-model="settings.theme">
            <option value="pink">ชมพู</option>
            <option value="blue">น้ำเงิน</option>
            <option value="green">เขียว</option>
            <option value="dark">ดำ</option>
          </select>

          <label>เป้าหมายยอดโดเนต (บาท)</label>
          <input v-model.number="settings.goal_amount" type="number" min="0" />

          <label>ระยะเวลาแสดงป็อบอัพ (วินาที)</label>
          <input v-model.number="settings.alert_duration_sec" type="number" min="3" max="30" />

          <label class="check">
            <input v-model="settings.tts_enabled" type="checkbox" />
            อ่านข้อความโดเนตด้วยเสียง (TTS)
          </label>

          <label>ข้อความแจ้งเตือน ({name} = ชื่อ, {amount} = ยอด)</label>
          <input v-model="settings.alert_text" maxlength="200" />

          <label>รูป/GIF ประกอบป็อบอัพ (URL https — เว้นว่าง = ไม่แสดง)</label>
          <input v-model="settings.alert_image_url" placeholder="https://example.com/cat.gif" />

          <label class="check">
            <input v-model="settings.show_leaderboard" type="checkbox" />
            แสดง Top Donators บนหน้าโดเนต
          </label>

          <label>เสียงแจ้งเตือนแบบกำหนดเอง (mp3/wav/ogg ≤ 2MB)</label>
          <div class="row">
            <input type="file" accept="audio/mpeg,audio/wav,audio/ogg" @change="(e) => (soundFile = (e.target as HTMLInputElement).files?.[0] ?? null)" />
            <button class="ghost" :disabled="!soundFile" @click="uploadSound">อัปโหลด</button>
            <audio v-if="soundUrl" :src="soundUrl" controls style="height: 34px" />
          </div>

          <div class="row actions">
            <button class="primary" @click="save">💾 บันทึก</button>
            <button class="ghost" @click="testAlert">▶ ทดสอบแจ้งเตือน</button>
            <a href="/overlay.html" target="_blank" class="ghost-link">เปิด Overlay (OBS)</a>
          </div>
          <p class="hint">ป็อบอัพจะใช้ธีม/เสียงใหม่ตั้งแต่โดเนตถัดไป — ทดสอบได้จากปุ่ม ▶</p>
        </section>
      </template>
    </main>
  </div>
</template>

<style scoped>
.page {
  min-height: 100vh;
  background: linear-gradient(160deg, var(--bg-0) 0%, var(--bg-1) 45%, #0f3460 100%);
  color: var(--text);
}
header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 28px;
  background: rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--border);
}
.logo {
  font-family: var(--font-head);
  font-weight: 700;
  font-size: 19px;
  background: linear-gradient(90deg, var(--accent-1), var(--accent-3));
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
.who { color: var(--text-dim); flex: 1; }
nav {
  display: flex;
  gap: 8px;
  padding: 14px 28px 0;
}
nav button {
  padding: 11px 20px;
  border: 1px solid transparent;
  border-radius: 12px 12px 0 0;
  background: rgba(0, 0, 0, 0.25);
  color: var(--text-dim);
  cursor: pointer;
  font-weight: 600;
  font-size: 14px;
  font-family: var(--font-body);
  transition: all 0.15s;
}
nav button:hover {
  color: var(--text);
}
nav button.active {
  background: var(--glass-strong);
  border-color: var(--border);
  border-bottom-color: transparent;
  color: var(--text);
  border-bottom: 3px solid var(--accent-1);
}
main {
  max-width: 780px;
  margin: 0 auto;
  padding: 22px 20px 48px;
  display: grid;
  gap: 18px;
}
.card {
  background: var(--glass);
  backdrop-filter: blur(16px);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 24px;
  box-shadow: var(--shadow-card);
}
h2 {
  font-size: 16px;
  margin-bottom: 14px;
  color: var(--text);
}
.stat-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}
.stat { text-align: center; padding: 20px 8px 16px; }
.stat .num {
  display: block;
  font-family: var(--font-head);
  font-size: 23px;
  font-weight: 700;
  color: var(--gold);
  font-variant-numeric: tabular-nums;
}
.stat span:last-child { font-size: 12px; color: var(--text-dim); }
label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-dim);
  margin: 12px 0 6px;
}
input, select {
  width: 100%;
  padding: 11px 13px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: rgba(0, 0, 0, 0.3);
  color: var(--text);
  font-size: 14px;
  font-family: var(--font-body);
  outline: none;
}
input:focus, select:focus {
  border-color: var(--accent-1);
  box-shadow: 0 0 0 3px rgba(255, 110, 199, 0.15);
}
input[type='checkbox'] { width: auto; }
.check { display: flex; align-items: center; gap: 8px; }
.link-row { display: flex; gap: 10px; }
.link-row code {
  flex: 1;
  background: rgba(0, 0, 0, 0.35);
  padding: 11px 13px;
  border-radius: var(--radius-sm);
  font-size: 13px;
  word-break: break-all;
  border: 1px solid var(--border);
}
.row { display: flex; gap: 10px; margin-top: 16px; align-items: center; flex-wrap: wrap; }
.actions { margin-top: 22px; }
button {
  padding: 10px 17px;
  border-radius: var(--radius-sm);
  border: none;
  cursor: pointer;
  font-weight: 600;
  font-size: 14px;
  font-family: var(--font-body);
  transition: transform 0.15s, background 0.15s;
}
button.primary {
  background: linear-gradient(135deg, var(--accent-1), var(--accent-2));
  color: #fff;
  box-shadow: 0 6px 20px rgba(255, 110, 199, 0.3);
}
button.primary:hover { transform: translateY(-1px); }
button.ghost { background: var(--glass-strong); color: var(--text); border: 1px solid var(--border); }
button.ghost:hover { background: rgba(255, 255, 255, 0.18); }
button.ghost:disabled { opacity: 0.5; }
button.mini { padding: 5px 11px; font-size: 12px; background: var(--glass-strong); color: var(--text); border: 1px solid var(--border); }
.ghost-link { color: #a5b4fc; font-size: 14px; }
.hint { font-size: 12px; color: var(--text-faint); margin-top: 10px; }
.toolbar { display: flex; gap: 10px; margin-bottom: 14px; align-items: center; }
table { width: 100%; border-collapse: collapse; font-size: 13.5px; }
th { text-align: left; color: var(--text-faint); font-weight: 600; padding: 9px 6px; border-bottom: 1px solid var(--border); }
td { padding: 9px 6px; border-bottom: 1px solid rgba(255, 255, 255, 0.05); }
td.amount { color: var(--gold); font-weight: 700; font-variant-numeric: tabular-nums; }
td.hidden { color: var(--text-faint); font-style: italic; }
.muted { color: var(--text-dim); }
.center { text-align: center; }
.pager { display: flex; gap: 12px; align-items: center; justify-content: center; margin-top: 12px; }
</style>
