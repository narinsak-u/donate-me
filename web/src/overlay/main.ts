// Overlay สำหรับ OBS Browser Source — standalone ไม่ผ่าน SPA router
// ใช้: /overlay.html?token=<JWT>[&w=all|alert|leaderboard|goal|recent][&pos=tl|tr|bl|br]
// ⚠️ ไม่ดึง token จาก localStorage (แท็บอื่นล็อกอินค้างจะทำให้ฟัง user ผิด)
//    แบบไม่ใส่ token = ฟังโดเนตของ streamer เริ่มต้น (ต้องเปิด ALLOW_PUBLIC_OVERLAY ใน .env)
// Phase 8: w= เลือกว่าจะ render วิดเจ็ตใด (all = เหมือนเดิม: alert+leaderboard)
//          ping ทุก 10 วิ → dashboard เห็น "วิดเจ็ตออนไลน์"

import { api, type DonationEvent, type Settings, type TopDonator } from '../api/auth'
import { playSound, speakThai } from '../sounds'
import { burstConfetti } from '../confetti'

const params = new URLSearchParams(location.search)
const token = params.get('token') ?? ''
// วิดเจ็ตที่จะแสดง: all | alert | leaderboard | goal | recent
const WIDGET = (() => {
  const w = (params.get('w') || 'all').toLowerCase()
  return ['all', 'alert', 'leaderboard', 'goal', 'recent'].includes(w) ? w : 'all'
})()
const show = (part: 'alert' | 'lb' | 'goal' | 'recent') => {
  if (WIDGET === 'all') return part === 'alert' || part === 'lb'
  return (WIDGET === 'leaderboard' ? 'lb' : WIDGET) === part
}

// มุมวางวิดเจ็ตย่อย (leaderboard/goal/recent) — tl|tr|bl|br, default ล่างซ้าย
const CORNER: Record<string, string> = {
  tl: 'top:22px; left:22px;',
  tr: 'top:22px; right:22px;',
  bl: 'bottom:22px; left:22px;',
  br: 'bottom:22px; right:22px;',
}
const corner = CORNER[params.get('pos') || ''] ?? CORNER.bl!

const WIDGET_CARD = `background:rgba(11,17,32,.82); backdrop-filter:blur(8px);
  border:1px solid rgba(148,163,184,.25); border-radius:14px; padding:12px 14px;
  font-family:'Noto Sans Thai',sans-serif; color:#e8eef9; position:fixed;`

// ---------- DOM ----------
const root = document.getElementById('app')!
root.innerHTML = `
  ${show('alert') ? `
  <div id="alert" style="
    position:fixed; left:50%; transform:translate(-50%,-50%) scale(0) rotate(-2deg);
    width:560px; padding:26px 30px 22px; border-radius:22px; text-align:left;
    background:linear-gradient(160deg, rgba(17,26,45,.94), rgba(11,17,32,.96));
    border:1.5px solid rgba(148,163,184,.28);
    box-shadow:0 24px 70px rgba(0,0,0,.6), 0 0 70px rgba(244,63,94,.28);
    opacity:0; transition:transform .55s cubic-bezier(.2,1.6,.4,1), opacity .3s, top .3s;
    font-family:'Noto Sans Thai','Segoe UI',sans-serif; color:#e8eef9;">
    <div style="position:absolute; inset:0; border-radius:22px; overflow:hidden; pointer-events:none;">
      <div id="a-shine" style="position:absolute; top:-60%; left:-80%; width:45%; height:220%;
        background:linear-gradient(100deg, transparent, rgba(255,255,255,.18), transparent);
        transform:rotate(15deg);"></div>
    </div>
    <div id="a-head" style="display:flex; align-items:center; justify-content:space-between; position:relative;">
      <span id="a-badge" style="display:inline-flex; align-items:center; gap:7px; padding:7px 16px;
        border-radius:999px; font-size:14px; font-weight:800; font-family:'Plus Jakarta Sans',sans-serif;
        background:linear-gradient(90deg,#fb7185,#f43f5e); color:#fff;">
        ❤️ DONATION
      </span>
      <span id="a-tts" style="display:none; align-items:center; gap:7px; padding:6px 13px; border-radius:999px;
        font-size:12px; font-weight:700; background:rgba(16,185,129,.15); color:#34d399;">🔊 TH-TTS</span>
    </div>
    <div style="display:flex; align-items:center; gap:16px; margin-top:18px; position:relative;">
      <div id="a-avatar" style="width:64px; height:64px; border-radius:50%; flex-shrink:0;
        background:linear-gradient(135deg,#f43f5e,#a855f7); display:flex; align-items:center;
        justify-content:center; font-size:26px; font-weight:800; color:#fff; border:3px solid rgba(255,255,255,.25);">💗</div>
      <div>
        <div style="display:flex; align-items:baseline; gap:10px; flex-wrap:wrap;">
          <span id="a-name" style="font-size:24px; font-weight:700; color:#fff;"></span>
          <span style="font-size:13px; color:#94a3b8;">ส่งกำลังใจ</span>
        </div>
        <div style="display:flex; align-items:center; gap:12px;">
          <span id="a-amount" style="font-size:42px; font-weight:800; font-family:'Plus Jakarta Sans',sans-serif;
            background:linear-gradient(90deg,#fde68a,#fbbf24); -webkit-background-clip:text; background-clip:text; color:transparent;"></span>
          <span id="a-tier" style="padding:4px 13px; border-radius:8px; font-size:12px; font-weight:800;
            letter-spacing:.5px; font-family:'Plus Jakarta Sans',sans-serif; background:rgba(251,191,36,.15); color:#fbbf24;">SUPPORT TIER</span>
        </div>
      </div>
    </div>
    <img id="a-img" alt="" style="display:none; max-width:100%; max-height:170px; border-radius:14px; margin-top:16px; position:relative;">
    <div id="a-msg-wrap" style="margin-top:16px; padding:14px 18px; border-radius:14px;
      background:rgba(148,163,184,.1); border:1px solid rgba(148,163,184,.15); position:relative;">
      <div id="a-msg" style="font-size:16px; line-height:1.6; color:#e2e8f0;"></div>
    </div>
    <div style="display:flex; align-items:center; justify-content:space-between; margin-top:14px; position:relative;">
      <span id="a-sound" style="font-size:12px; color:#94a3b8;">♪ เสียงประกอบ</span>
      <span style="font-size:12px; color:#f43f5e; font-weight:700;">● Donate Me ❤️ Engine</span>
    </div>
  </div>` : ''}
  ${show('lb') ? `
  <!-- Leaderboard widget -->
  <div id="lb" style="${WIDGET_CARD} ${WIDGET === 'all' ? 'bottom:22px; left:22px;' : corner} width:230px; display:none;">
    <div style="font-size:12px; font-weight:800; color:#fbbf24; margin-bottom:8px;">🏆 TOP DONATORS</div>
    <ol id="lb-list" style="list-style:none; display:grid; gap:5px;"></ol>
  </div>` : ''}
  ${show('goal') ? `
  <!-- Goal widget -->
  <div id="goal" style="${WIDGET_CARD} ${corner} width:280px;">
    <div style="display:flex; justify-content:space-between; font-size:12px; font-weight:800; margin-bottom:8px;">
      <span style="color:#34d399;">🎯 เป้าหมายเดือนนี้</span>
      <span id="g-pct" style="color:#fff;">0%</span>
    </div>
    <div style="height:10px; border-radius:999px; background:rgba(148,163,184,.2); overflow:hidden;">
      <div id="g-fill" style="height:100%; width:0%; border-radius:999px;
        background:linear-gradient(90deg,#34d399,#10b981); transition:width .8s ease;"></div>
    </div>
    <div style="display:flex; justify-content:space-between; margin-top:7px; font-size:11.5px; color:#94a3b8;">
      <span id="g-raised">฿0</span><span id="g-target">เป้า ฿0</span>
    </div>
  </div>` : ''}
  ${show('recent') ? `
  <!-- Recent donations widget -->
  <div id="recent" style="${WIDGET_CARD} ${corner} width:270px;">
    <div style="font-size:12px; font-weight:800; color:#fb7185; margin-bottom:8px;">⚡ กำลังใจล่าสุด</div>
    <ol id="r-list" style="list-style:none; display:grid; gap:6px;"></ol>
  </div>` : ''}
  <style>
    @keyframes bounce { from { transform:translateY(0); } to { transform:translateY(-8px); } }
    @keyframes shine { from { left:-80%; } to { left:160%; } }
    .lb-row { display:flex; align-items:center; gap:7px; font-size:12px; }
    .lb-row .lb-n { flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:#e2e8f0; }
    .lb-row .lb-t { color:#fbbf24; font-weight:700; }
    .r-row { display:flex; align-items:center; gap:7px; font-size:12px; }
    .r-row .r-n { flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:#e2e8f0; }
    .r-row .r-t { color:#34d399; font-weight:700; }
    .confetti { position:fixed; top:-30px; pointer-events:none; animation:fall 3s linear forwards; z-index:-1; }
    @keyframes fall { to { transform:translateY(105vh) rotate(720deg); opacity:0; } }
  </style>
`

document.body.style.cssText = 'margin:0;height:100%;background:rgba(0,0,0,0);overflow:hidden;font-family:Segoe UI,Noto Sans Thai,sans-serif'

const alertBox = root.querySelector<HTMLElement>('#alert')
const elBadge = root.querySelector<HTMLElement>('#a-badge')
const elTts = root.querySelector<HTMLElement>('#a-tts')
const elAvatar = root.querySelector<HTMLElement>('#a-avatar')
const elImg = root.querySelector<HTMLImageElement>('#a-img')
const elName = root.querySelector<HTMLElement>('#a-name')
const elAmount = root.querySelector<HTMLElement>('#a-amount')
const elTier = root.querySelector<HTMLElement>('#a-tier')
const elMsg = root.querySelector<HTMLElement>('#a-msg')
const elSound = root.querySelector<HTMLElement>('#a-sound')
const elShine = root.querySelector<HTMLElement>('#a-shine')
const lbBox = root.querySelector<HTMLElement>('#lb')
const lbList = root.querySelector<HTMLElement>('#lb-list')
const goalBox = root.querySelector<HTMLElement>('#goal')
const gFill = root.querySelector<HTMLElement>('#g-fill')
const gPct = root.querySelector<HTMLElement>('#g-pct')
const gRaised = root.querySelector<HTMLElement>('#g-raised')
const gTarget = root.querySelector<HTMLElement>('#g-target')
const recentBox = root.querySelector<HTMLElement>('#recent')
const recentList = root.querySelector<HTMLElement>('#r-list')

function playShine() {
  if (!elShine) return
  elShine.style.animation = 'none'
  void elShine.offsetWidth // restart animation
  elShine.style.animation = 'shine 1.1s ease-out .35s'
}

// fetch ที่แนบ token จาก URL (overlay ไม่อ่าน localStorage — ต่างจาก client ใน SPA)
async function authFetch<T>(path: string): Promise<T> {
  const res = await fetch(path, {
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  })
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json() as Promise<T>
}

// ---------- settings ----------
let settings: Settings = {
  theme: 'pink',
  goal_amount: 0,
  alert_duration_sec: 6,
  tts_enabled: true,
  alert_text: 'ขอบคุณ {name} ที่โดเนต {amount} บาท',
  alert_sound_url: '',
  alert_image_url: '',
  show_leaderboard: true,
  alert_position: 'middle',
  tts_speed: 1.0,
  tts_max_len: 120,
  tier_vip_amount: 100,
  tier_gold_amount: 500,
  promptpay_id: '',
  bank_name: '',
  bank_no: '',
  page_theme: 'rose',
  cover_url: '',
  about_text: '',
  social_facebook: '',
  social_youtube: '',
  social_twitch: '',
  social_tiktok: '',
  social_x: '',
}

async function loadSettings() {
  if (!token) return
  try {
    settings = await api.getSettings()
  } catch {
    /* ใช้ค่า default ถ้าดึงไม่ได้ */
  }
}

const POSITIONS: Record<string, string> = { top: '20%', middle: '44%', bottom: '68%' }
function applyPosition() {
  if (alertBox) alertBox.style.top = POSITIONS[settings.alert_position] ?? POSITIONS.middle
}

// ธีมสี accent ของ badge และพื้นหลังการ์ด ตาม settings.theme
function applyTheme() {
  const themes: Record<string, { c1: string; c2: string; bg: string }> = {
    pink: { c1: '#fb7185', c2: '#f43f5e', bg: 'linear-gradient(160deg, rgba(45,17,35,.94), rgba(17,10,25,.96))' },
    blue: { c1: '#60a5fa', c2: '#3b82f6', bg: 'linear-gradient(160deg, rgba(15,30,55,.94), rgba(8,15,32,.96))' },
    green: { c1: '#34d399', c2: '#10b981', bg: 'linear-gradient(160deg, rgba(10,38,30,.94), rgba(6,20,16,.96))' },
    dark: { c1: '#94a3b8', c2: '#64748b', bg: 'linear-gradient(160deg, rgba(17,26,45,.94), rgba(11,17,32,.96))' },
  }
  const t = themes[settings.theme] ?? themes.pink!
  if (elBadge) elBadge.style.background = `linear-gradient(90deg, ${t.c1}, ${t.c2})`
  if (alertBox) alertBox.style.background = t.bg
}

// ---------- tier ตามยอด (เกณฑ์มาจาก settings) ----------
function tierOf(amount: number): {
  scale: number; badge: string; tier: string; tierColor: string; avatar: string;
  confettiCount: number; soundLabel: string;
} {
  if (amount >= settings.tier_gold_amount)
    return { scale: 1.22, badge: '❤️ SUPER CHAT DONATION', tier: 'GOLD TIER', tierColor: '#fbbf24', avatar: '👑', confettiCount: 80, soundLabel: '♪ Sound: Level_Up_Super.mp3' }
  if (amount >= settings.tier_vip_amount)
    return { scale: 1.08, badge: '⭐ VIP DONATION', tier: 'VIP TIER', tierColor: '#a78bfa', avatar: '💜', confettiCount: 50, soundLabel: '♪ Sound: Vip_Glow.mp3' }
  return { scale: 1, badge: '❤️ DONATION', tier: 'SUPPORT TIER', tierColor: '#34d399', avatar: '💗', confettiCount: 30, soundLabel: '♪ Sound: Standard_Support.mp3' }
}

// ---------- ป็อบอัพ ----------
let hideTimer: ReturnType<typeof setTimeout> | undefined

function showAlert(d: DonationEvent) {
  if (!alertBox || !elBadge || !elTts || !elAvatar || !elImg || !elName || !elAmount || !elTier || !elMsg || !elSound) return
  clearTimeout(hideTimer)
  const tier = tierOf(d.amount)
  // กัน XSS: ใช้ textContent เท่านั้น ห้าม innerHTML กับข้อมูลจากผู้ใช้
  elName.textContent = d.donor_name
  elAmount.textContent = `฿${d.amount.toLocaleString()}`
  elAvatar.textContent = tier.avatar
  elMsg.textContent = d.message
  elBadge.textContent = tier.badge
  elTier.textContent = tier.tier
  elTier.style.color = tier.tierColor
  elTier.style.background = `${tier.tierColor}26`
  elSound.textContent = tier.soundLabel
  elTts.style.display = settings.tts_enabled ? 'inline-flex' : 'none'
  // media alert: รูป/GIF จาก settings (https เท่านั้น ตรวจแล้วฝั่ง server)
  if (settings.alert_image_url) {
    elImg.src = settings.alert_image_url
    elImg.style.display = 'block'
  } else {
    elImg.style.display = 'none'
  }
  applyPosition()
  applyTheme()
  alertBox.style.transform = 'translate(-50%,-50%) scale(0) rotate(-2deg)'
  requestAnimationFrame(() => {
    alertBox.style.transform = `translate(-50%,-50%) scale(${tier.scale}) rotate(0deg)`
    alertBox.style.opacity = '1'
  })
  playShine()
  burstConfetti(tier.confettiCount, undefined, { behind: true })

  // เสียง: เสียงที่เลือก/อัปโหลดนำ แล้วอ่านข้อความผู้บริจาคตามทุกครั้ง (ถ้าเปิด tts_enabled)
  if (settings.alert_sound_url) {
    const audio = new Audio(settings.alert_sound_url)
    void audio.play().catch(() => playSound(d.sound))
  } else {
    playSound(d.sound)
  }
  if (settings.tts_enabled) {
    const text = settings.alert_text
      .replaceAll('{name}', d.donor_name)
      .replaceAll('{amount}', d.amount.toLocaleString())
    // ข้อความเริ่มต้นเมื่อผู้บริจาคไม่พิมพ์ — ตรงกับหน้าโดเนต (useDonationWatch)
    const msg = (d.message || 'เป็นกำลังใจให้นะ').slice(0, settings.tts_max_len)
    setTimeout(() => void speakThai(`${text}. ${msg}`, settings.tts_speed), 1400)
  }

  hideTimer = setTimeout(() => {
    alertBox.style.transform = 'translate(-50%,-50%) scale(0) rotate(-2deg)'
    alertBox.style.opacity = '0'
  }, settings.alert_duration_sec * 1000)
}

// ---------- Leaderboard widget ----------
async function loadLeaderboard() {
  if (!lbBox || !lbList) return
  try {
    const top: TopDonator[] = await api.leaderboard()
    if (!settings.show_leaderboard || top.length === 0) {
      lbBox.style.display = 'none'
      return
    }
    lbList.innerHTML = top
      .map(
        (t, i) =>
          `<li class="lb-row"><span>${['🥇', '🥈', '🥉'][i] ?? '#' + (i + 1)}</span><span class="lb-n"></span><span class="lb-t">฿${t.total.toLocaleString()}</span></li>`,
      )
      .join('')
    // ชื่อใส่แบบ textContent (กัน XSS) — เติมทีหลัง innerHTML ของโครง
    const names = lbList.querySelectorAll<HTMLElement>('.lb-n')
    top.forEach((t, i) => {
      if (names[i]) names[i]!.textContent = t.donor_name
    })
    lbBox.style.display = 'block'
  } catch {
    lbBox.style.display = 'none'
  }
}

// ---------- Goal widget (Phase 8) ----------
async function loadGoal() {
  if (!goalBox || !gFill || !gPct || !gRaised || !gTarget) return
  try {
    const s = await authFetch<{ goal_amount: number; total_month: number }>('/api/me/stats')
    if (s.goal_amount <= 0) {
      goalBox.style.display = 'none'
      return
    }
    const pct = Math.min(100, Math.round((s.total_month / s.goal_amount) * 100))
    gFill.style.width = pct + '%'
    gPct.textContent = pct + '%'
    gRaised.textContent = `฿${s.total_month.toLocaleString()}`
    gTarget.textContent = `เป้า ฿${s.goal_amount.toLocaleString()}`
    goalBox.style.display = 'block'
  } catch {
    goalBox.style.display = 'none'
  }
}

// ---------- Recent donations widget (Phase 8) ----------
async function loadRecent() {
  if (!recentBox || !recentList) return
  try {
    const res = await authFetch<{ items: { donor_name: string; amount: number; paid_at: number | null }[] }>(
      '/api/me/donations?per_page=5',
    )
    const items = res.items
    if (items.length === 0) {
      recentBox.style.display = 'none'
      return
    }
    recentList.innerHTML = items
      .map(
        (it) =>
          `<li class="r-row"><span>💗</span><span class="r-n"></span><span class="r-t">฿${it.amount.toLocaleString()}</span></li>`,
      )
      .join('')
    const names = recentList.querySelectorAll<HTMLElement>('.r-n')
    items.forEach((it, i) => {
      if (names[i]) names[i]!.textContent = it.donor_name
    })
    recentBox.style.display = 'block'
  } catch {
    recentBox.style.display = 'none'
  }
}

// ---------- SSE + ping + auto-reconnect ----------
function pingLoop() {
  // แจ้ง server ว่าวิดเจ็ตยังเปิดอยู่ (TTL 30 วิ — ping ทุก 10 วิ)
  const p = token ? `token=${encodeURIComponent(token)}&` : ''
  void fetch(`/api/me/widget-ping?${p}w=${WIDGET}`, { method: 'POST' }).catch(() => {})
}
setInterval(pingLoop, 10_000)

function connect() {
  const url = token ? `/events?token=${encodeURIComponent(token)}` : '/events'
  const es = new EventSource(`${url}&w=${WIDGET}`.replace(/&$/, ''))
  es.addEventListener('donation', (e) => {
    const d = JSON.parse((e as MessageEvent).data) as DonationEvent
    if (show('alert')) showAlert(d)
    // ทุกวิดเจ็ตย่อยรีเฟรชข้อมูลตามโดเนตใหม่
    void loadLeaderboard()
    void loadGoal()
    void loadRecent()
  })
  es.onerror = () => {
    es.close()
    setTimeout(connect, 3000)
  }
}

void (async () => {
  await loadSettings()
  applyTheme()
  applyPosition()
  await Promise.all([loadLeaderboard(), loadGoal(), loadRecent()])
  pingLoop()
  connect()
})()

// ทดสอบจาก OBS ได้: เรียก window.testAlert() ใน console ของ Browser Source
declare global {
  interface Window {
    testAlert: () => void
  }
}
window.testAlert = () =>
  showAlert({ user_id: '', id: 'test', donor_name: 'คุณทดสอบ', amount: 250, message: 'นี่คือการทดสอบป็อบอัพ', sound: 'chime' })
