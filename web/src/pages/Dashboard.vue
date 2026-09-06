<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api, clearSession, getToken, getUser, request, type PublicUser, type Settings } from '../api/auth'
import { dashApi, type DonationItem, type SlipItem, type Stats } from '../api/dashboard'
import { applyTheme } from '../theme'
import DonationChart from '../components/DonationChart.vue'
import WalletTab from '../components/WalletTab.vue'

const router = useRouter()
const route = useRoute()
const user = ref<PublicUser | null>(getUser())
const tab = ref<'overview' | 'donations' | 'slips' | 'settings' | 'wallet'>('overview')
const donateLink = ref('')
const copied = ref(false)
const overlayUrl = ref('')
const obsCopied = ref(false)

const stats = ref<Stats | null>(null)
const items = ref<DonationItem[]>([])
const search = ref('')
const dateFrom = ref('')
const dateTo = ref('')
const page = ref(1)
const totalPages = ref(1)
const settings = ref<Settings | null>(null)
const soundFile = ref<File | null>(null)
const soundUrl = ref('')
const loadError = ref('')

// ---------- Phase 7: บัญชีรับเงินโอนตรง + คิวสลิป ----------
const slips = ref<SlipItem[]>([])
const slipBusy = ref('') // id ที่กำลังกดอนุมัติ/ปฏิเสธ
const rejectTarget = ref<SlipItem | null>(null)
const rejectNote = ref('')
const slipsCount = computed(() => slips.value.filter((s) => s.status === 'awaiting_review').length)

async function loadSlips() {
  try {
    slips.value = await dashApi.slips()
  } catch {
    /* โหลดคิวไม่ได้ไม่กระทบแท็บอื่น */
  }
}

async function approve(s: SlipItem) {
  slipBusy.value = s.id
  try {
    await dashApi.approveSlip(s.id)
    s.status = 'paid'
  } catch (e) {
    alert(e instanceof Error ? e.message : 'อนุมัติไม่สำเร็จ')
  } finally {
    slipBusy.value = ''
  }
}

function openReject(s: SlipItem) {
  rejectTarget.value = s
  rejectNote.value = ''
}

async function confirmReject() {
  const s = rejectTarget.value
  if (!s) return
  slipBusy.value = s.id
  try {
    await dashApi.rejectSlip(s.id, rejectNote.value)
    s.status = 'rejected'
    s.review_note = rejectNote.value
    rejectTarget.value = null
  } catch (e) {
    alert(e instanceof Error ? e.message : 'ปฏิเสธไม่สำเร็จ')
  } finally {
    slipBusy.value = ''
  }
}

/// ตั้งค่าบัญชีรับเงินโอนตรงในแท็บ settings ทำงานผ่าน save() เดิม (settings promptpay_id ฯลฯ)
const testQrAmount = ref(20)
const ppValid = computed(() => /^(0\d{9}|\d{13})$/.test(settings.value?.promptpay_id.replace(/[-\s]/g, '') ?? ''))

// ---------- Phase 8: สถานะวิดเจ็ต OBS ----------
const widgetStatus = ref<{ online: boolean; widgets: string[] } | null>(null)

async function loadWidgetStatus() {
  try {
    widgetStatus.value = await request<{ online: boolean; widgets: string[] }>('/api/me/widget-status')
  } catch {
    /* โหลดไม่ได้ = ยังเช็คไม่ได้ */
  }
}

const WIDGET_KINDS = [
  { w: 'all', label: 'ครบทุกวิดเจ็ต', desc: 'Alert + กระดาน (แบบเดิม)' },
  { w: 'alert', label: 'ป็อบอัพ Alert', desc: 'เฉพาะป็อบอัพ + เสียง' },
  { w: 'leaderboard', label: 'กระดาน Top', desc: 'วางมุมจอได้ด้วย ?pos=tl/tr/bl/br' },
  { w: 'goal', label: 'หลอดเป้าหมาย', desc: 'อัปเดตสดตามโดเนต' },
  { w: 'recent', label: 'กำลังใจล่าสุด', desc: '5 รายการหลังสุด' },
] as const

function widgetUrl(w: string): string {
  return `${location.origin}/overlay.html?token=${encodeURIComponent(getToken() ?? '')}&w=${w}`
}

const copiedW = ref('')
async function copyWidgetUrl(w: string) {
  await navigator.clipboard.writeText(widgetUrl(w))
  copiedW.value = w
  setTimeout(() => (copiedW.value = ''), 2000)
}

onMounted(async () => {
  applyTheme()
  // เปิดแท็บตาม ?tab= (เช่น ลิงก์ "คู่มือสตรีมเมอร์" → ?tab=settings)
  const q = route.query.tab
  if (typeof q === 'string' && ['overview', 'donations', 'slips', 'settings', 'wallet'].includes(q)) {
    tab.value = q as typeof tab.value
  }
  if (!getToken()) {
    router.push('/auth')
    return
  }
  if (user.value) {
    donateLink.value = `${location.origin}/u/${user.value.username}`
    overlayUrl.value = `${location.origin}/overlay.html?token=${getToken() ?? ''}`
  }
  // โหลดอิสระจากกัน — stats ล่มอย่างเดียวต้องไม่ทำให้แท็บตั้งค่าว่าง (และกลับกัน)
  const [s, st] = await Promise.allSettled([dashApi.stats(), api.getSettings()])
  if (s.status === 'fulfilled') stats.value = s.value
  if (st.status === 'fulfilled') settings.value = st.value
  const failReason: unknown =
    s.status === 'rejected' ? s.reason : st.status === 'rejected' ? st.reason : undefined
  if (failReason) {
    const failed = failReason as Error & { status?: number }
    // token ชี้บัญชีที่ไม่มีอยู่แล้ว (เช่น DB ถูก seed ใหม่) → พาไปล็อกอินใหม่
    if (failed.status === 401 || failed.status === 404) {
      clearSession()
      router.push('/auth')
      return
    }
    loadError.value = 'โหลดข้อมูล Dashboard ไม่สำเร็จ — ลองรีเฟรชหน้าอีกครั้ง'
  }
  try {
    await loadHistory()
  } catch {
    /* ประวัติโหลดไม่ได้ไม่ถือว่าทั้งหน้าพัง */
  }
  void loadSlips()
  void loadWidgetStatus()
  // คิวสลิป: ดึงใหม่ทุก 20 วิ เมื่ออยู่แท็บคิว (มีคนแนบสลิปเข้ามาเอง)
  setInterval(() => {
    if (tab.value === 'slips') void loadSlips()
    void loadWidgetStatus()
  }, 20_000)
})

async function loadHistory() {
  const toMillis = (d: string, end = false) => {
    if (!d) return undefined
    const t = new Date(d).getTime()
    return end ? t + 86_399_999 : t
  }
  const res = await dashApi.history({
    query: search.value,
    page: page.value,
    from: toMillis(dateFrom.value),
    to: toMillis(dateTo.value, true),
  })
  items.value = res.items
  totalPages.value = res.total_pages
}

function copyLink() {
  navigator.clipboard.writeText(donateLink.value)
  copied.value = true
  setTimeout(() => (copied.value = false), 2000)
}

function copyOverlayUrl() {
  navigator.clipboard.writeText(overlayUrl.value)
  obsCopied.value = true
  setTimeout(() => (obsCopied.value = false), 2000)
}

async function toggleHidden(item: DonationItem) {
  await dashApi.setHidden(item.id, !item.hidden)
  item.hidden = !item.hidden
}

async function togglePinned(item: DonationItem) {
  await dashApi.setPinned(item.id, !item.pinned)
  item.pinned = !item.pinned
  await loadHistory() // pinned เรียงขึ้นบน
}

const csvUrl = computed(() => {
  const toMillis = (d: string, end = false) => {
    if (!d) return undefined
    const t = new Date(d).getTime()
    return end ? t + 86_399_999 : t
  }
  return dashApi.csvUrl(toMillis(dateFrom.value), toMillis(dateTo.value, true))
})

async function save() {
  if (!settings.value) return
  try {
    settings.value = await api.updateSettings(settings.value)
    alert('บันทึกแล้ว!')
  } catch (e) {
    alert(e instanceof Error ? e.message : 'บันทึกไม่สำเร็จ')
  }
}

// Phase 9: ธีมหน้าโดเนต
const pageThemes = [
  { value: 'rose', label: 'Rose (เดิม)', desc: 'ชมพู Stitch', color: '#f43f5e' },
  { value: 'mint', label: 'Mint', desc: 'เขียวสดใส', color: '#10b981' },
  { value: 'midnight', label: 'Midnight', desc: 'ม่วงน้ำเงินลึก', color: '#818cf8' },
  { value: 'retro', label: 'Retro', desc: 'เหลืองสายวินเทจ', color: '#f59e0b' },
]

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

const testSending = ref(false)
const testDone = ref(false)
const testError = ref('')

async function testAlert(amount = 100) {
  testSending.value = true
  testDone.value = false
  testError.value = ''
  try {
    await api.testAlert(amount)
    // alert เด้งใน overlay — ทั้งใน OBS และพรีวิวสดในการ์ดนี้
    testDone.value = true
    setTimeout(() => (testDone.value = false), 4000)
  } catch (e) {
    testError.value = e instanceof Error ? e.message : 'ส่งคำสั่งทดสอบไม่สำเร็จ'
  } finally {
    testSending.value = false
  }
}

function logout() {
  localStorage.removeItem('donateme_token')
  localStorage.removeItem('donateme_user')
  router.push('/auth')
}

function fmtDate(ms: number | null): string {
  if (!ms) return '-'
  return new Date(ms).toLocaleString('th-TH', { dateStyle: 'short', timeStyle: 'short' })
}

function soundLabel(s: string): string {
  const map: Record<string, string> = {
    chime: '🔔 กระดิ่ง',
    coin: '🪙 เหรียญ',
    fanfare: '🎺 แตรวง',
    tts: '🗣️ TTS',
  }
  return map[s] ?? s
}

const total30 = computed(() => stats.value?.series_30d.reduce((s, d) => s + d.total, 0) ?? 0)
const avgPerBill = computed(() =>
  stats.value && stats.value.count_all > 0 ? Math.round(stats.value.total_all / stats.value.count_all) : 0,
)
const goalPct = computed(() =>
  stats.value && stats.value.goal_amount > 0
    ? Math.min(100, Math.round((stats.value.total_month / stats.value.goal_amount) * 100))
    : 0,
)

const navItems = [
  { key: 'overview', icon: '📊', label: 'ภาพรวม (Dashboard)' },
  { key: 'donations', icon: '🧾', label: 'ประวัติโดเนต' },
  { key: 'slips', icon: '🏦', label: 'ตั้งค่าบัญชี' },
  { key: 'settings', icon: '🔔', label: 'ตั้งค่าแจ้งเตือน (Alerts)' },
  { key: 'wallet', icon: '💰', label: 'กระเป๋าเงิน & ถอน' },
] as const

const themeOptions = [
  { value: 'pink', label: 'Cyber Pink', desc: 'ชมพูสายเกมเก่ง', color: '#f43f5e' },
  { value: 'blue', label: 'Stream Blue', desc: 'น้ำเงินคนชิลล์', color: '#3b82f6' },
  { value: 'green', label: 'Matrix Emerald', desc: 'เขียวสายมินิมอล', color: '#10b981' },
  { value: 'dark', label: 'Stealth Dark', desc: 'เทาสายโปรดักช์', color: '#64748b' },
]
</script>

<template>
  <div class="shell">
    <!-- ===== Sidebar ===== -->
    <aside class="sidebar">
      <div class="side-logo" role="link" title="กลับหน้าหลัก" @click="router.push('/')">
        <div class="side-logo-icon">❤️</div>
        <div>
          <b>Donate Me</b>
          <span>CREATOR STUDIO</span>
        </div>
      </div>

      <div class="caster-box">
        <span class="caster-url">donateme.live/caster</span>
        <button class="icon-btn" title="คัดลอก">⧉</button>
      </div>

      <nav class="side-nav">
        <button
          v-for="n in navItems"
          :key="n.key"
          :class="{ active: tab === n.key }"
          @click="tab = n.key; n.key === 'slips' && loadSlips()"
        >
          <i>{{ n.icon }}</i> {{ n.label }}
          <span v-if="n.key === 'slips' && slipsCount" class="slip-badge">{{ slipsCount }}</span>
        </button>
      </nav>

      <div class="obs-box">
        <span class="obs-label">OBS BROWSER SOURCE</span>
        <span class="obs-ready">● READY</span>
        <code>obs://localhost:3000/overlay</code>
      </div>

      <button class="side-footer" @click="logout">⚙ ออกจากระบบ</button>
    </aside>

    <!-- ===== Main ===== -->
    <div class="main">
      <header class="topbar">
        <span class="live-pill">● LIVE ON AIR</span>
        <span
          class="widget-pill"
          :class="widgetStatus?.online ? 'on' : 'off'"
          :title="widgetStatus?.widgets.join(', ') || 'ยังไม่มีการเชื่อมต่อ'"
        >
          {{ widgetStatus?.online ? '🟢 OBS ออนไลน์' : '🔴 วิดเจ็ตออฟไลน์' }}
        </span>
        <span class="today-pill">ยอดสะสมวันนี้ <b>฿{{ (stats?.total_today ?? 0).toLocaleString() }}.00</b></span>
        <div class="topbar-right">
          <button class="btn-primary test-btn" @click="testAlert()">▶ ทดสอบ Alert</button>
          <div class="profile">
            <div class="profile-avatar">{{ user?.display_name?.[0] ?? '?' }}</div>
            <div class="profile-info">
              <b>{{ user?.display_name }}</b>
              <span>Partner</span>
            </div>
          </div>
        </div>
      </header>

      <main class="content">
        <p v-if="loadError" class="load-error" role="alert">{{ loadError }}</p>
        <!-- ===== Overview ===== -->
        <template v-if="tab === 'overview' && stats">
          <section class="welcome">
            <div class="welcome-avatar">{{ user?.display_name?.[0] ?? '?' }}</div>
            <div class="welcome-info">
              <h1>ยินดีต้อนรับกลับมา, {{ user?.display_name }}! 👋</h1>
              <p><i class="dot green" /> กำลังสตรีมมิ่ง · <b>เว็บรับโดเนตของคุณ</b> · <span>⏱ ออนไลน์ต่อเนื่อง 3 ชม. 42 นาที</span></p>
            </div>
            <div class="welcome-link">
              <span>🔗 {{ donateLink }}</span>
              <button class="btn-ghost" @click="copyLink">{{ copied ? '✓ คัดลอกแล้ว' : '⧉ คัดลอก' }}</button>
            </div>
          </section>

          <div class="metrics">
            <div class="metric">
              <div class="metric-head"><span>รายได้วันนี้ (TODAY)</span><i class="m-icon pink">💰</i></div>
              <b class="metric-num">฿{{ stats.total_today.toLocaleString() }}<small>.00</small></b>
              <span class="metric-foot"><em class="grow">↗ สดใหม่</em> อัปเดตอัตโนมัติ</span>
            </div>
            <div class="metric">
              <div class="metric-head"><span>ยอดเดือนนี้ (MONTH)</span><i class="m-icon green">📅</i></div>
              <b class="metric-num">฿{{ stats.total_month.toLocaleString() }}<small>.00</small></b>
              <span class="metric-foot">🧾 {{ stats.count_all }} รายการทั้งหมด</span>
            </div>
            <div class="metric">
              <div class="metric-head"><span>ยอดสะสมทั้งหมด (ALL-TIME)</span><i class="m-icon gold">🏆</i></div>
              <b class="metric-num">฿{{ stats.total_all.toLocaleString() }}</b>
              <span class="metric-foot">สะสม 30 วัน ฿{{ total30.toLocaleString() }}</span>
            </div>
            <div class="metric">
              <div class="metric-head"><span>เฉลี่ยต่อบิล (AVG. TIP)</span><i class="m-icon blue">📈</i></div>
              <b class="metric-num">฿{{ avgPerBill.toLocaleString() }}<small>.50</small></b>
              <span class="metric-foot"><em class="grow">↗ กำลังวิ่งขึ้น</em> ต่อรายการ</span>
            </div>
          </div>

          <section class="goal-card">
            <div class="goal-left">
              <div class="goal-title-row">
                <i class="goal-flag">🚩</i>
                <div>
                  <b>เป้าหมาย: อัปเกรดอุปกรณ์สตรีม</b>
                  <span>รวบรวมทุกการสนับสนุนเพื่อเป้าหมายถัดไปของช่อง</span>
                </div>
                <span class="badge badge-green">กำลังดำเนินการ</span>
              </div>
              <div class="goal-bar"><div class="goal-fill" :style="{ width: goalPct + '%' }" /></div>
              <div class="goal-foot">
                <span>ปัจจุบัน: <b class="green">฿{{ stats.total_month.toLocaleString() }}</b></span>
                <span>เป้าหมาย: <b>฿{{ stats.goal_amount.toLocaleString() }}</b></span>
              </div>
            </div>
            <div class="goal-right">
              <span class="goal-remain">ต้องการอีก</span>
              <b>{{ goalPct }}%</b>
              <span class="muted small">จากเป้าหมาย</span>
            </div>
          </section>

          <section class="card">
            <div class="card-head">
              <h2>📈 สถิติรายได้ 30 วันล่าสุด</h2>
              <span class="badge badge-pink">รายวัน</span>
            </div>
            <DonationChart :data="stats.series_30d" :goal="stats.goal_amount" />
          </section>
        </template>

        <!-- ===== Donations ===== -->
        <template v-if="tab === 'donations'">
          <section class="card">
            <div class="card-head">
              <h2>🧾 ประวัติการโดนตล่าสุด</h2>
              <a :href="csvUrl" class="export-btn">⬇ Export CSV</a>
            </div>
            <div class="toolbar">
              <input v-model="search" class="input" placeholder="ค้นหาชื่อ/ข้อความ..." @keyup.enter="page = 1; loadHistory()" />
              <input v-model="dateFrom" class="input date" type="date" title="จากวันที่" @change="page = 1; loadHistory()" />
              <span class="muted">ถึง</span>
              <input v-model="dateTo" class="input date" type="date" title="ถึงวันที่" @change="page = 1; loadHistory()" />
              <button class="btn-ghost" @click="page = 1; loadHistory()">ค้นหา</button>
              <button v-if="dateFrom || dateTo || search" class="mini" @click="search = ''; dateFrom = ''; dateTo = ''; page = 1; loadHistory()">✕ ล้างตัวกรอง</button>
            </div>
            <table>
              <thead>
                <tr><th>เวลา</th><th>ผู้ส่งกำลังใจ</th><th>ยอดเงิน</th><th>ข้อความ</th><th>เสียง</th><th>จัดการ</th></tr>
              </thead>
              <tbody>
                <tr v-for="it in items" :key="it.id">
                  <td class="muted">{{ fmtDate(it.paid_at) }}</td>
                  <td><b>{{ it.donor_name }}</b> <span v-if="it.pinned" title="ปักหมุด">📌</span></td>
                  <td class="amount">฿{{ it.amount.toLocaleString() }}</td>
                  <td :class="{ hidden: it.hidden }">{{ it.hidden ? '(ซ่อนแล้ว)' : it.message || '-' }}</td>
                  <td><span class="sound-pill">{{ soundLabel(it.sound) }}</span></td>
                  <td class="row-actions">
                    <button class="mini" @click="togglePinned(it)">{{ it.pinned ? '📌 เลิกปัก' : '📌 ปักหมุด' }}</button>
                    <button class="mini" @click="toggleHidden(it)">{{ it.hidden ? 'แสดง' : 'ซ่อน' }}</button>
                  </td>
                </tr>
                <tr v-if="!items.length"><td colspan="6" class="muted center">ยังไม่มีประวัติโดเนตในช่วงนี้</td></tr>
              </tbody>
            </table>
            <div class="pager" v-if="totalPages > 1">
              <button class="mini" :disabled="page <= 1" @click="page--; loadHistory()">← ก่อนหน้า</button>
              <span class="muted">{{ page }} / {{ totalPages }}</span>
              <button class="mini" :disabled="page >= totalPages" @click="page++; loadHistory()">ถัดไป →</button>
            </div>
          </section>
        </template>

        <!-- ===== ตั้งค่าบัญชี (บัญชีรับเงินโอนตรง + คิวสลิป) ===== -->
        <template v-if="tab === 'slips'">
          <section v-if="settings" class="card">
            <div class="card-head">
              <h2>🏦 บัญชีรับเงิน (โอนตรง 0% ค่าธรรมเนียม)</h2>
              <span class="badge" :class="ppValid ? 'badge-green' : 'badge-pink'">
                {{ ppValid ? 'เปิดรับโอนตรงแล้ว' : 'ยังไม่เปิดใช้' }}
              </span>
            </div>
            <p class="muted small" style="margin-bottom: 12px">
              กรอกเบอร์พร้อมเพย์ — หน้าโดเนตจะสร้าง QR จริงให้ผู้ชมสแกนโอน<b>เข้าบัญชีคุณโดยตรง</b> (เงินไม่ผ่านเว็บ)
              ผู้ชมแนบสลิป → คุณกดอนุมัติในคิวด้านล่าง → Alert เด้งบนจอ
              <template v-if="!ppValid"> · <b>ไม่กรอก = ใช้ QR จำลอง (โหมดทดสอบ) เหมือนเดิม</b></template>
            </p>
            <div class="tts-opts">
              <div>
                <label>เบอร์พร้อมเพย์ (10 หลัก) หรือเลขบัตร (13 หลัก)</label>
                <input
                  v-model="settings.promptpay_id"
                  class="input"
                  inputmode="numeric"
                  placeholder="0812345678"
                  maxlength="13"
                />
              </div>
              <div>
                <label>ธนาคาร (ช่องทางสำรอง — ไม่บังคับ)</label>
                <input v-model="settings.bank_name" class="input" placeholder="ธนาคารไทยพาณิชย์" maxlength="60" />
              </div>
              <div>
                <label>เลขบัญชี</label>
                <input v-model="settings.bank_no" class="input" inputmode="numeric" placeholder="4051234567" maxlength="30" />
              </div>
            </div>
            <div v-if="ppValid" class="pp-test">
              <div class="pp-qr">
                <img
                  :key="testQrAmount"
                  :src="dashApi.promptpayQrUrl(testQrAmount)"
                  alt="QR ทดสอบพร้อมเพย์ของคุณ"
                  width="150"
                  height="150"
                />
                <span class="muted small">QR จริงของคุณ (ทดลองสแกนได้)</span>
              </div>
              <div class="pp-test-side">
                <label>ยอดทดสอบ (฿)</label>
                <input v-model.number="testQrAmount" type="number" min="1" max="100000" class="input" style="max-width: 130px" />
                <p class="muted small">เปลี่ยนยอดแล้ว QR จะรีเฟรช — ลองสแกนด้วยแอปธนาคารก่อนเปิดใช้จริง</p>
              </div>
            </div>
          </section>

          <section class="card">
            <div class="card-head">
              <h2>🧿 คิวสลิป — โอนเข้าบัญชีคุณตรง</h2>
              <span class="badge" :class="slipsCount ? 'badge-pink' : 'badge-green'">
                {{ slipsCount ? `รอตรวจ ${slipsCount} รายการ` : 'ไม่มีค้างตรวจ' }}
              </span>
            </div>
            <p class="muted" style="margin-bottom: 14px">
              ผู้ชมโอนเงินเข้าพร้อมเพย์ของคุณโดยตรง (เงินไม่ผ่านเว็บ) แล้วแนบสลิป — ตรวจกับแอปธนาคารแล้วกดอนุมัติ Alert จะเด้งบนจอทันที
            </p>

            <div v-if="slips.length" class="slip-queue">
              <div v-for="s in slips" :key="s.id" class="slip-row" :class="{ done: s.status !== 'awaiting_review' }">
                <a v-if="s.has_image" :href="dashApi.slipImageUrl(s.id)" target="_blank" class="slip-thumb" title="เปิดรูปสลิปเต็ม">
                  <img :src="dashApi.slipImageUrl(s.id)" alt="รูปสลิป" loading="lazy" />
                </a>
                <div v-else class="slip-thumb empty">–</div>
                <div class="slip-info">
                  <b>{{ s.donor_name }}</b>
                  <span class="slip-msg">{{ s.message || '(ไม่มีข้อความ)' }}</span>
                  <span class="muted small">แนบสลิป {{ s.slip_at ? fmtDate(s.slip_at) : '-' }} · เสียง {{ soundLabel(s.sound) }}</span>
                </div>
                <b class="slip-amount">฿{{ s.amount.toLocaleString() }}</b>
                <div v-if="s.status === 'awaiting_review'" class="row-actions">
                  <button class="btn-primary" :disabled="slipBusy === s.id" @click="approve(s)">
                    {{ slipBusy === s.id ? '...' : '✓ อนุมัติ' }}
                  </button>
                  <button class="btn-ghost" :disabled="slipBusy === s.id" @click="openReject(s)">✕ ปฏิเสธ</button>
                </div>
                <div v-else class="slip-verdict" :class="s.status === 'paid' ? 'ok' : 'no'">
                  {{ s.status === 'paid' ? '✓ อนุมัติแล้ว' : '✕ ปฏิเสธ' }}
                  <small v-if="s.review_note">{{ s.review_note }}</small>
                </div>
              </div>
            </div>
            <p v-else class="muted center" style="padding: 28px 0">
              ยังไม่มีสลิปเข้า — เมื่อผู้ชมโอนตรงและแนบสลิป รายการจะขึ้นที่นี่
            </p>
          </section>

          <!-- กล่องยืนยันการปฏิเสธ -->
          <section v-if="rejectTarget" class="card reject-box">
            <h2>ปฏิเสธสลิปของ {{ rejectTarget.donor_name }} (฿{{ rejectTarget.amount.toLocaleString() }})</h2>
            <p class="muted small" style="margin: 6px 0 12px">เหตุผลนี้ผู้ชมจะเห็นบนหน้าจ่ายเงิน — เช่น "ยอดไม่ตรง" / "ไม่พบรายการโอน"</p>
            <input v-model="rejectNote" class="input" maxlength="200" placeholder="ระบุเหตุผล (ไม่บังคับ)" @keyup.enter="confirmReject" />
            <div class="save-row" style="margin-top: 12px">
              <button class="btn-primary" :disabled="slipBusy === rejectTarget.id" @click="confirmReject">ยืนยันปฏิเสธ</button>
              <button class="btn-ghost" @click="rejectTarget = null">ยกเลิก</button>
            </div>
          </section>
        </template>

        <!-- ===== Wallet ===== -->
        <template v-if="tab === 'wallet'">
          <WalletTab />
        </template>

        <!-- ===== Settings ===== -->
        <template v-if="tab === 'settings' && settings">
          <section class="card">
            <div class="card-head">
              <h2>📺 OBS Browser Source Integration</h2>
              <span class="badge badge-green">1920×1080 · 60fps</span>
            </div>
            <div class="obs-url-row">
              <code class="obs-url">{{ overlayUrl }}</code>
              <button class="btn-primary" @click="copyOverlayUrl">{{ obsCopied ? '✓ คัดลอกแล้ว' : '⧉ คัดลอก URL สำหรับ OBS' }}</button>
            </div>
            <div class="steps-3">
              <div class="step"><b>Step 1</b><span>เพิ่ม Browser Source ใน OBS Studio</span></div>
              <div class="step"><b>Step 2</b><span>วาง URL ข้างบน ตั้งขนาด 1920×1080</span></div>
              <div class="step"><b>Step 3</b><span>กดทดสอบ Alert แล้วดูผลในฉากจริง</span></div>
            </div>
            <p class="muted small" style="margin-top: 10px">🔒 อย่าแชร์ URL นี้กับคนอื่น — token ผูกกับบัญชีของคุณ (หมดอายุ 7 วัน ล็อกอินใหม่เพื่อรีเฟรช)</p>
          </section>

          <section class="card">
            <div class="card-head">
              <h2>🧩 วิดเจ็ตแยกชิ้น (ใส่ OBS ทีละช่อง)</h2>
              <span class="badge" :class="widgetStatus?.online ? 'badge-green' : 'badge-pink'">
                {{ widgetStatus?.online ? `ออนไลน์: ${widgetStatus.widgets.join(', ')}` : 'ยังไม่เชื่อมต่อ' }}
              </span>
            </div>
            <p class="muted small" style="margin-bottom: 12px">
              เพิ่มหลาย Browser Source ได้ — แต่ละ URL แสดงเฉพาะวิดเจ็ตของตัวเอง วางมุมจอด้วย <code>&amp;pos=tl|tr|bl|br</code> (สถานะเขียว = OBS เปิด URL นั้นอยู่ ตรวจทุก 30 วิ)
            </p>
            <table class="widget-table">
              <thead><tr><th>วิดเจ็ต</th><th>URL สำหรับ OBS</th><th></th></tr></thead>
              <tbody>
                <tr v-for="k in WIDGET_KINDS" :key="k.w">
                  <td><b>{{ k.label }}</b><br /><span class="muted small">{{ k.desc }}</span></td>
                  <td><code class="widget-url">{{ widgetUrl(k.w) }}</code></td>
                  <td><button class="mini" @click="copyWidgetUrl(k.w)">{{ copiedW === k.w ? '✓ คัดลอกแล้ว' : '⧉ คัดลอก' }}</button></td>
                </tr>
              </tbody>
            </table>
          </section>

          <section class="card">
            <div class="card-head">
              <h2>🎨 Theme & Visual Styling</h2>
              <span class="badge badge-pink">ป็อบอัพแจ้งเตือน</span>
            </div>
            <label>เลือกชุดสีธีม (THEME PALETTE)</label>
            <div class="palettes">
              <button
                v-for="t in themeOptions"
                :key="t.value"
                class="palette"
                :class="{ active: settings.theme === t.value }"
                @click="settings.theme = t.value"
              >
                <i class="swatch" :style="{ background: t.color }" />
                <b>{{ t.label }}</b>
                <span>{{ t.desc }}</span>
              </button>
            </div>
            <label>ระยะเวลาแสดงผล (DURATION)</label>
            <div class="dur-row">
              <input v-model.number="settings.alert_duration_sec" type="range" min="3" max="30" class="range" />
              <b class="dur-val">{{ settings.alert_duration_sec }} วินาที</b>
            </div>

            <label>ตำแหน่งบนจอ (POSITION)</label>
            <select v-model="settings.alert_position">
              <option value="top">บนจอ</option>
              <option value="middle">กลางจอ</option>
              <option value="bottom">ล่างจอ</option>
            </select>

            <label>เกณฑ์ระดับยอด (ALERT TIERS)</label>
            <div class="tier-config">
              <div>
                <span class="t-label t20">SUPPORT</span>
                <span class="muted small">ต่ำกว่า {{ settings.tier_vip_amount.toLocaleString() }} ฿</span>
              </div>
              <div>
                <label>฿ VIP ตั้งแต่</label>
                <input v-model.number="settings.tier_vip_amount" type="number" min="1" class="input" />
              </div>
              <div>
                <label>฿ GOLD ตั้งแต่</label>
                <input v-model.number="settings.tier_gold_amount" type="number" min="1" class="input" />
              </div>
            </div>
          </section>

          <section class="card">
            <div class="card-head">
              <h2>🔔 ทดสอบ Alert</h2>
              <span class="badge badge-green">OBS LINKED</span>
            </div>
            <p class="muted">กดปุ่มเพื่อทดสอบป็อบอัพตามระดับยอด — เด้งทั้งพรีวิวด้านล่างและบน OBS ทันที</p>
            <div class="tiers">
              <button class="tier" :disabled="testSending" @click="testAlert(20)">
                <b class="tier-amt t20">฿20</b>
                <div><b>Alert ฝั่งปกติ ฿20</b><span>ป็อปปกติ + คอนเฟตติสี</span></div>
                <i class="play">▶</i>
              </button>
              <button class="tier" :disabled="testSending" @click="testAlert(100)">
                <b class="tier-amt t100">฿100</b>
                <div><b>Alert ระดับกลาง ฿100+</b><span>แอนิเมชันพิเศษ + ข้อความ TTS เต็มรูปแบบ</span></div>
                <i class="play">▶</i>
              </button>
              <button class="tier gold" :disabled="testSending" @click="testAlert(500)">
                <b class="tier-amt t500">฿500+</b>
                <div><b>Super Chat ฿500+</b><span>ทองคำ + TTS เร่งเสียง + คอนเฟตติลูกใหญ่</span></div>
                <i class="play">▶</i>
              </button>
            </div>
            <p v-if="testDone" class="test-status ok" role="status">✓ ส่งแล้ว — alert กำลังเด้งในพรีวิวด้านล่าง (และบน OBS ถ้าเปิดไว้)</p>
            <p v-else-if="testError" class="test-status err" role="alert">{{ testError }}</p>

            <div class="preview-shell">
              <div class="preview-head">🔍 พรีวิวสด — เหมือนใน OBS</div>
              <iframe v-if="overlayUrl" :src="overlayUrl" class="preview-frame" title="พรีวิว Overlay"></iframe>
              <p v-else class="muted small preview-loading">กำลังโหลดพรีวิว...</p>
            </div>

            <p class="muted small" style="margin-top: 14px">Latency: ~120ms · <a href="/overlay.html" target="_blank" style="color: var(--primary)">เปิดหน้า Overlay ↗</a></p>
          </section>

          <section class="card">
            <div class="card-head">
              <h2>🔊 Sound & Thai AI Voice (TTS)</h2>
            </div>
            <label class="check">
              <input v-model="settings.tts_enabled" type="checkbox" />
              <span>เปิดใช้งานการอ่านออกเสียงข้อความไทย (Thai TTS)</span>
            </label>
            <p class="muted small" style="margin-top: 6px">ระบบจะอ่านข้อความโดเนตออกเสียงไทยในฉากสตรีมทันทีที่ยอดรับเข้ามา</p>

            <div class="tts-opts">
              <div>
                <label>ความเร็วเสียงอ่าน (SPEED)</label>
                <div class="dur-row">
                  <input v-model.number="settings.tts_speed" type="range" min="0.5" max="2" step="0.1" class="range" />
                  <b class="dur-val">{{ settings.tts_speed.toFixed(1) }}x</b>
                </div>
              </div>
              <div>
                <label>จำกัดความยาวอ่าน (ตัวอักษร)</label>
                <input v-model.number="settings.tts_max_len" type="number" min="20" max="200" class="input" />
              </div>
            </div>

            <label>เสียงแจ้งเตือนแบบกำหนดเอง (mp3/wav/ogg ≤ 2MB)</label>
            <div class="upload-row">
              <input type="file" accept="audio/mpeg,audio/wav,audio/ogg" @change="(e) => (soundFile = (e.target as HTMLInputElement).files?.[0] ?? null)" />
              <button class="btn-primary" :disabled="!soundFile" @click="uploadSound">อัปโหลด</button>
              <audio v-if="soundUrl" :src="soundUrl" controls style="height: 34px" />
            </div>

            <label>ข้อความแจ้งเตือน (MESSAGE FORMAT)</label>
            <input v-model="settings.alert_text" class="input" maxlength="200" />
            <p class="muted small" style="margin-top: 6px">ตัวแปร: <code>{name}</code> = ชื่อผู้โดเนต · <code>{amount}</code> = ยอดเงิน</p>

            <div class="save-row">
              <button class="btn-primary" @click="save">💾 บันทึกการตั้งค่า</button>
              <button class="btn-ghost" @click="testAlert()">▶ ทดสอบแจ้งเตือน</button>
              <a href="/overlay.html" target="_blank" class="ghost-link">เปิดหน้า Overlay (OBS) ↗</a>
            </div>
          </section>

          <section class="card">
            <div class="card-head">
              <h2>⚙️ อื่น ๆ</h2>
            </div>
            <label>เป้าหมายยอดโดเนต (บาท)</label>
            <input v-model.number="settings.goal_amount" class="input" type="number" min="0" />
            <label class="check" style="margin-top: 14px">
              <input v-model="settings.show_leaderboard" type="checkbox" />
              <span>แสดง Leaderboard Top 5 บนหน้าโดเนต</span>
            </label>
            <label>รูป/GIF ประกอบป็อบอัพ (URL https — เว้นว่าง = ไม่แสดง)</label>
            <input v-model="settings.alert_image_url" class="input" placeholder="https://example.com/cat.gif" />
          </section>

          <section class="card">
            <div class="card-head">
              <h2>🎨 หน้าโดเนตของฉัน</h2>
              <span class="badge badge-pink">เห็นผลทันที →</span>
            </div>
            <label>ธีมหน้า (PAGE THEME)</label>
            <div class="palettes">
              <button
                v-for="t in pageThemes"
                :key="t.value"
                class="palette"
                :class="{ active: settings.page_theme === t.value }"
                @click="settings.page_theme = t.value"
              >
                <i class="swatch" :style="{ background: t.color }" />
                <b>{{ t.label }}</b>
                <span>{{ t.desc }}</span>
              </button>
            </div>

            <label>รูปปกหน้าโดเนต (URL https — เว้นว่าง = ใช้พื้นหลังเดิม)</label>
            <input v-model="settings.cover_url" class="input" placeholder="https://images.example.com/banner.jpg" />

            <label>เกี่ยวกับฉัน (แสดงใต้ปก — ไม่เกิน 400 ตัวอักษร)</label>
            <textarea v-model="settings.about_text" class="input" rows="3" maxlength="400" placeholder="สวัสดีครับ ผมสตรีมเกม ทุกคืน 20:00 💜" />

            <label>ลิงก์โซเชียล (https — เว้นว่างช่องไหน = ไม่แสดง)</label>
            <div class="tts-opts">
              <div><label>f Facebook</label><input v-model="settings.social_facebook" class="input" placeholder="https://facebook.com/ชื่อคุณ" /></div>
              <div><label>▶ YouTube</label><input v-model="settings.social_youtube" class="input" placeholder="https://youtube.com/@ชื่อคุณ" /></div>
              <div><label>🎮 Twitch</label><input v-model="settings.social_twitch" class="input" placeholder="https://twitch.tv/ชื่อคุณ" /></div>
              <div><label>♪ TikTok</label><input v-model="settings.social_tiktok" class="input" placeholder="https://tiktok.com/@ชื่อคุณ" /></div>
              <div><label>𝕏 (Twitter)</label><input v-model="settings.social_x" class="input" placeholder="https://x.com/ชื่อคุณ" /></div>
            </div>

            <p class="muted small" style="margin-top: 14px">
              💡 ดูผลจริงได้ที่ลิงก์หน้าโดเนตของคุณ: <a :href="donateLink" target="_blank" rel="noopener" style="color: var(--primary)">{{ donateLink }} ↗</a>
            </p>
          </section>
        </template>
      </main>
    </div>
  </div>
</template>

<style scoped>
.shell { display: flex; min-height: 100vh; background: var(--bg); }

/* sidebar */
.sidebar {
  width: 250px; flex-shrink: 0; display: flex; flex-direction: column; gap: 14px;
  padding: 18px 14px; border-right: 1px solid var(--border);
  background: var(--bg-card);
  position: sticky; top: 0; height: 100vh;
}
.side-logo { display: flex; align-items: center; gap: 11px; padding: 4px 8px; cursor: pointer; user-select: none; }
.side-logo:hover b { color: var(--primary); }
.side-logo-icon {
  width: 40px; height: 40px; border-radius: 13px; font-size: 18px;
  background: linear-gradient(135deg, #fb7185, var(--primary));
  display: flex; align-items: center; justify-content: center;
  box-shadow: 0 6px 18px rgba(244, 63, 94, 0.4);
}
.side-logo b { display: block; font-family: var(--font-head); font-size: 16px; }
.side-logo span { display: block; font-size: 9.5px; letter-spacing: 1.5px; color: var(--text-faint); font-weight: 700; }
.caster-box {
  display: flex; align-items: center; justify-content: space-between;
  padding: 9px 12px; border-radius: var(--radius-md);
  background: var(--bg-card-2); border: 1px solid var(--border);
}
.caster-url { font-size: 12px; color: var(--text-dim); }
.icon-btn { border: none; background: none; color: var(--text-dim); cursor: pointer; font-size: 14px; }
.side-nav { display: flex; flex-direction: column; gap: 4px; }
.side-nav button {
  display: flex; align-items: center; gap: 10px;
  padding: 11px 14px; border-radius: var(--radius-md); border: none;
  background: transparent; color: var(--text-dim);
  font-size: 13.5px; font-weight: 600; font-family: var(--font-body);
  cursor: pointer; text-align: left; transition: all 0.15s;
}
.side-nav button:hover { color: var(--text); background: var(--hover-row); }
.side-nav button.active {
  background: linear-gradient(135deg, #fb7185, var(--primary));
  color: #fff; box-shadow: 0 6px 18px rgba(244, 63, 94, 0.35);
}
.side-nav button i { font-style: normal; font-size: 15px; }
.obs-box {
  margin-top: auto; padding: 12px; border-radius: var(--radius-md);
  background: var(--bg-card-2); border: 1px solid var(--border);
  display: flex; flex-direction: column; gap: 3px;
}
.obs-label { font-size: 10px; letter-spacing: 1px; color: var(--text-faint); font-weight: 700; }
.obs-ready { font-size: 11px; color: var(--emerald); font-weight: 700; }
.obs-box code { font-size: 10.5px; color: var(--text-dim); word-break: break-all; }
.side-footer {
  border: none; background: none; color: var(--text-dim); text-align: left;
  font-size: 13px; font-weight: 600; cursor: pointer; padding: 10px 8px 0;
  font-family: var(--font-body);
}
.side-footer:hover { color: var(--primary); }

/* main */
.main { flex: 1; min-width: 0; }
.topbar {
  display: flex; align-items: center; gap: 14px;
  padding: 13px 26px; border-bottom: 1px solid var(--border);
  background: color-mix(in srgb, var(--bg) 80%, transparent);
  backdrop-filter: blur(12px);
  position: sticky; top: 0; z-index: 10;
}
.live-pill {
  padding: 7px 15px; border-radius: 999px; font-size: 12px; font-weight: 800;
  background: var(--primary-soft); color: var(--primary); letter-spacing: 0.5px;
}
.today-pill {
  display: inline-flex; align-items: center; gap: 8px;
  padding: 7px 15px; border-radius: 999px; font-size: 12.5px; color: var(--text-dim);
  border: 1px solid var(--border);
}
.today-pill b { color: var(--emerald); font-family: var(--font-head); }
.topbar-right { margin-left: auto; display: flex; align-items: center; gap: 14px; }
.test-btn { padding: 10px 20px; font-size: 13.5px; }
.profile { display: flex; align-items: center; gap: 10px; }
.profile-avatar {
  width: 38px; height: 38px; border-radius: 50%; overflow: hidden;
  background: linear-gradient(135deg, var(--primary), #a855f7);
  display: flex; align-items: center; justify-content: center;
  font-weight: 800; color: #fff; font-size: 15px;
}
.profile-info b { display: block; font-size: 13px; }
.profile-info span { display: block; font-size: 11px; color: var(--emerald); }

.content { padding: 24px 28px 48px; display: grid; gap: 18px; max-width: 1220px; }
.load-error {
  padding: 14px 18px;
  border-radius: var(--radius-md);
  background: var(--primary-soft);
  color: var(--primary);
  font-size: 13.5px;
  font-weight: 600;
}

/* welcome */
.welcome {
  display: flex; align-items: center; gap: 16px;
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius-xl); padding: 20px 24px; box-shadow: var(--shadow-card);
}
.welcome-avatar {
  width: 56px; height: 56px; border-radius: 17px;
  background: linear-gradient(135deg, var(--primary), #a855f7);
  display: flex; align-items: center; justify-content: center;
  font-size: 24px; font-weight: 800; color: #fff;
}
.welcome-info { flex: 1; }
.welcome-info h1 { font-size: 21px; }
.welcome-info p { font-size: 13px; color: var(--text-dim); margin-top: 4px; display: flex; align-items: center; gap: 7px; }
.welcome-info b { color: var(--text); }
.dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.dot.green { background: var(--emerald); box-shadow: 0 0 8px var(--emerald); }
.welcome-link {
  display: flex; align-items: center; gap: 10px;
  padding: 9px 10px 9px 16px; border-radius: var(--radius-md);
  background: var(--bg-card-2); border: 1px solid var(--border);
}
.welcome-link span { font-size: 13px; color: var(--text-dim); }
.welcome-link .btn-ghost { padding: 8px 14px; }

/* metrics */
.metrics { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; }
.metric {
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius-lg); padding: 18px; box-shadow: var(--shadow-card);
}
.metric-head { display: flex; justify-content: space-between; align-items: center; font-size: 11.5px; color: var(--text-faint); font-weight: 700; letter-spacing: 0.4px; }
.m-icon {
  width: 34px; height: 34px; border-radius: 11px; font-style: normal;
  display: flex; align-items: center; justify-content: center; font-size: 15px;
}
.m-icon.pink { background: var(--primary-soft); }
.m-icon.green { background: var(--emerald-soft); }
.m-icon.gold { background: rgba(251, 191, 36, 0.14); }
.m-icon.blue { background: rgba(59, 130, 246, 0.14); }
.metric-num {
  display: block; font-family: var(--font-head); font-size: 29px; font-weight: 800;
  margin: 12px 0 6px; font-variant-numeric: tabular-nums;
}
.metric-num small { font-size: 15px; color: var(--text-faint); }
.metric-foot { font-size: 11.5px; color: var(--text-faint); display: flex; align-items: center; gap: 7px; }
.grow { font-style: normal; color: var(--emerald); font-weight: 700; background: var(--emerald-soft); padding: 2px 8px; border-radius: 6px; }

/* goal */
.goal-card {
  display: flex; gap: 24px; align-items: center;
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius-xl); padding: 22px 24px; box-shadow: var(--shadow-card);
}
.goal-left { flex: 1; }
.goal-title-row { display: flex; align-items: center; gap: 12px; margin-bottom: 14px; }
.goal-flag {
  width: 40px; height: 40px; border-radius: 12px; font-style: normal;
  background: rgba(251, 191, 36, 0.14); display: flex; align-items: center; justify-content: center;
}
.goal-title-row b { display: block; font-size: 15px; }
.goal-title-row span { display: block; font-size: 12px; color: var(--text-dim); }
.goal-bar { height: 10px; border-radius: 999px; background: var(--border); overflow: hidden; }
.goal-fill { height: 100%; border-radius: 999px; background: linear-gradient(90deg, var(--primary), var(--gold)); transition: width 0.5s; }
.goal-foot { display: flex; justify-content: space-between; margin-top: 9px; font-size: 12.5px; color: var(--text-dim); }
.goal-foot .green { color: var(--emerald); }
.goal-right { text-align: center; border-left: 1px solid var(--border); padding-left: 24px; }
.goal-right b { display: block; font-family: var(--font-head); font-size: 36px; color: var(--gold); }

/* cards */
.card {
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius-xl); padding: 22px; box-shadow: var(--shadow-card);
}
.grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 18px; align-items: start; }
.card-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; }
.card-head h2 { font-size: 16px; }
.muted { color: var(--text-dim); font-size: 12.5px; }
.small { font-size: 11.5px; }
label { display: block; font-size: 13px; font-weight: 600; color: var(--text-dim); margin: 14px 0 7px; }
input, select { font-family: var(--font-body); }
.check { display: flex; align-items: center; gap: 9px; cursor: pointer; }
.check input { width: auto; accent-color: var(--primary); }

/* tiers */
.tiers { display: grid; gap: 9px; margin-top: 12px; }
.test-status { margin-top: 12px; padding: 10px 14px; border-radius: var(--radius-md); font-size: 13px; font-weight: 600; }
.test-status.ok { background: var(--emerald-soft); color: var(--emerald); }
.test-status.err { background: var(--primary-soft); color: var(--primary); }
.preview-shell {
  margin-top: 16px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--border);
  background:
    linear-gradient(45deg, rgba(148, 163, 184, 0.06) 25%, transparent 25%, transparent 75%, rgba(148, 163, 184, 0.06) 75%) 0 0 / 22px 22px,
    linear-gradient(45deg, rgba(148, 163, 184, 0.06) 25%, transparent 25%, transparent 75%, rgba(148, 163, 184, 0.06) 75%) 11px 11px / 22px 22px,
    #0b0f1a;
  overflow: hidden;
}
.preview-head {
  padding: 8px 14px;
  font-size: 11.5px;
  font-weight: 700;
  color: #94a3b8;
  background: rgba(148, 163, 184, 0.08);
  border-bottom: 1px solid rgba(148, 163, 184, 0.15);
}
.preview-frame { display: block; width: 100%; height: 260px; border: none; background: transparent; }
.preview-loading { padding: 40px; text-align: center; }
.tier {
  display: flex; align-items: center; gap: 14px; padding: 12px 14px;
  border-radius: var(--radius-md); border: 1px solid var(--border);
  background: var(--bg-input); color: var(--text); cursor: pointer;
  text-align: left; font-family: var(--font-body); transition: border 0.15s, transform 0.15s;
}
.tier:hover { border-color: var(--primary); transform: translateX(3px); }
.tier.gold { border-color: rgba(251, 191, 36, 0.5); }
.tier-amt {
  padding: 6px 11px; border-radius: 9px; font-family: var(--font-head); font-size: 13px;
}
.t20 { background: var(--primary-soft); color: var(--primary); }
.t100 { background: rgba(167, 139, 250, 0.15); color: #a78bfa; }
.t500 { background: rgba(251, 191, 36, 0.15); color: var(--gold); }
.tier div { flex: 1; }
.tier div b { display: block; font-size: 13.5px; }
.tier div span { display: block; font-size: 11.5px; color: var(--text-dim); }
.play { font-style: normal; color: var(--text-faint); }

/* table */
.toolbar { display: flex; gap: 10px; margin-bottom: 14px; }
.export-btn {
  padding: 9px 18px; border-radius: var(--radius-md); text-decoration: none;
  background: var(--emerald); color: #fff; font-size: 13px; font-weight: 700;
}
table { width: 100%; border-collapse: collapse; font-size: 13.5px; }
th { text-align: left; color: var(--text-faint); font-weight: 700; padding: 9px 8px; border-bottom: 1px solid var(--border); }
td { padding: 10px 8px; border-bottom: 1px solid var(--hover-row); }
td.amount { color: var(--gold); font-weight: 700; font-family: var(--font-head); font-variant-numeric: tabular-nums; }
td.hidden { color: var(--text-faint); font-style: italic; }
.center { text-align: center; padding: 24px; }
.pager { display: flex; gap: 12px; align-items: center; justify-content: center; margin-top: 12px; }
.mini { padding: 6px 12px; font-size: 12px; background: var(--glass-strong, var(--bg-card-2)); color: var(--text); border: 1px solid var(--border); }

/* settings */
.palettes { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }
.palette {
  display: flex; flex-direction: column; align-items: center; gap: 6px;
  padding: 16px 10px; border-radius: var(--radius-lg);
  border: 1.5px solid var(--border); background: var(--bg-input);
  color: var(--text); cursor: pointer; font-family: var(--font-body);
  transition: all 0.15s;
}
.palette:hover { border-color: var(--border-bright); }
.palette.active { border-color: var(--primary); box-shadow: 0 0 0 3px var(--primary-soft); }
.swatch { width: 30px; height: 30px; border-radius: 50%; }
.palette b { font-size: 12.5px; }
.palette span { font-size: 10.5px; color: var(--text-faint); }
.dur-row { display: flex; align-items: center; gap: 14px; }
.range { flex: 1; accent-color: var(--primary); }
.dur-val { font-size: 13px; color: var(--primary); background: var(--primary-soft); padding: 5px 12px; border-radius: 8px; }
.tier-config { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 12px; align-items: end; }
.t-label { font-size: 12px; font-weight: 800; padding: 4px 12px; border-radius: 8px; display: inline-block; }
.t-label.t20 { background: var(--primary-soft); color: var(--primary); }
.tts-opts { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-top: 6px; }
.row-actions { display: flex; gap: 6px; }
.upload-row { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
.upload-row input[type='file'] { font-size: 12.5px; color: var(--text-dim); }
.save-row { display: flex; gap: 10px; margin-top: 20px; align-items: center; flex-wrap: wrap; }
.ghost-link { color: #a5b4fc; font-size: 13.5px; }
code { background: var(--bg-card-2); padding: 1px 6px; border-radius: 5px; font-size: 11.5px; color: var(--primary); }

/* obs url card */
.obs-url-row { display: flex; gap: 10px; align-items: stretch; }
.obs-url {
  flex: 1; display: flex; align-items: center;
  background: var(--bg-input); border: 1px solid var(--border);
  padding: 12px 14px; border-radius: var(--radius-md);
  font-size: 12.5px; color: var(--text-dim); word-break: break-all;
}
.steps-3 { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; margin-top: 14px; }
.step {
  display: flex; align-items: center; gap: 10px;
  padding: 12px 13px; border-radius: var(--radius-md);
  background: var(--bg-card-2); border: 1px solid var(--border);
  font-size: 12px; color: var(--text-dim);
}
.step b {
  padding: 3px 9px; border-radius: 7px; font-size: 10.5px; white-space: nowrap;
  background: var(--emerald-soft); color: var(--emerald);
}

@media (max-width: 1000px) {
  .metrics { grid-template-columns: 1fr 1fr; }
  .grid-2 { grid-template-columns: 1fr; }
  .goal-card { flex-direction: column; }
  .goal-right { border: none; padding: 0; }
  .palettes { grid-template-columns: 1fr 1fr; }
  .steps-3 { grid-template-columns: 1fr; }
}

/* mobile: sidebar → top nav */
@media (max-width: 780px) {
  .shell { flex-direction: column; }
  .sidebar {
    position: static; height: auto; width: 100%;
    flex-direction: row; flex-wrap: wrap; align-items: center;
    border-right: none; border-bottom: 1px solid var(--border);
  }
  .side-logo { width: 100%; }
  .caster-box { flex: 1; }
  .side-nav { flex-direction: row; flex-wrap: wrap; width: 100%; }
  .side-nav button { padding: 9px 12px; font-size: 12.5px; }
  .obs-box { display: none; }
  .side-footer { margin-left: auto; padding: 0; }
  .topbar { flex-wrap: wrap; padding: 10px 16px; }
  .content { padding: 16px 14px 40px; }
  .welcome { flex-direction: column; align-items: flex-start; }
  .welcome-link { width: 100%; }
  .welcome-link span { font-size: 11.5px; word-break: break-all; }
  .toolbar { flex-wrap: wrap; }
  .toolbar .date { width: auto; }
  .obs-url-row { flex-direction: column; }
  .withdraw-form { grid-template-columns: 1fr !important; }
  table { font-size: 12px; }
  td, th { padding: 7px 4px; }
}

/* withdraw form */
.withdraw-form { display: grid; grid-template-columns: 2fr 1fr 1fr auto; gap: 12px; align-items: end; margin-top: 8px; }
.withdraw-form label { margin-top: 10px; }
.error { color: var(--primary); font-size: 13px; margin-top: 12px; }
.sound-pill {
  font-size: 11.5px; padding: 3px 10px; border-radius: 999px;
  background: var(--bg-card-2); border: 1px solid var(--border); color: var(--text-dim);
}
.status-pill { font-size: 12px; font-weight: 700; }
.status-pill.pending { color: var(--gold); }
.status-pill.completed { color: var(--emerald); }
.status-pill.rejected { color: var(--primary); }
.toolbar .date { width: auto; padding: 10px 12px; }

/* ===== Phase 7: คิวสลิป + บัญชีรับเงิน ===== */
.widget-pill {
  font-size: 12.5px;
  font-weight: 700;
  padding: 6px 13px;
  border-radius: 999px;
}
.widget-pill.on { background: rgba(16, 185, 129, 0.12); color: var(--emerald); }
.widget-pill.off { background: rgba(244, 63, 94, 0.12); color: var(--primary); }
.widget-table td { padding: 9px 10px; }
.widget-url { font-size: 11px; color: var(--text-dim); word-break: break-all; }
.slip-badge {
  margin-left: auto;
  background: var(--primary);
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  min-width: 20px;
  height: 20px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0 6px;
}
.slip-queue { display: grid; gap: 10px; }
.slip-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--bg-card);
}
.slip-row.done { opacity: 0.72; }
.slip-thumb {
  width: 54px;
  height: 70px;
  border-radius: 8px;
  overflow: hidden;
  flex-shrink: 0;
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-faint);
  cursor: zoom-in;
}
.slip-thumb img { width: 100%; height: 100%; object-fit: cover; }
.slip-thumb.empty { font-size: 20px; }
.slip-info { display: flex; flex-direction: column; gap: 3px; min-width: 0; flex: 1; }
.slip-info b { font-size: 14.5px; }
.slip-msg { font-size: 13px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.slip-amount { color: var(--gold); font-size: 17px; font-weight: 700; font-family: var(--font-head); }
.slip-row .row-actions { display: flex; gap: 8px; }
.slip-verdict { display: flex; flex-direction: column; font-size: 13px; text-align: right; }
.slip-verdict.ok { color: var(--emerald); }
.slip-verdict.no { color: var(--primary); }
.slip-verdict small { color: var(--text-faint); font-size: 11.5px; max-width: 200px; }
.reject-box { border-color: rgba(244, 63, 94, 0.4); }
.pp-test { display: flex; gap: 20px; margin-top: 16px; align-items: flex-start; flex-wrap: wrap; }
.pp-qr {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
  background: #fff;
  padding: 10px;
  border-radius: var(--radius-md);
  border: 1px solid #e2e8f0;
}
.pp-qr img { width: 150px; height: 150px; image-rendering: pixelated; }
.pp-test-side { display: flex; flex-direction: column; gap: 8px; max-width: 260px; }

@media (max-width: 640px) {
  .slip-row { flex-wrap: wrap; }
  .slip-amount { order: 3; }
}
</style>
