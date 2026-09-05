// Overlay สำหรับ OBS Browser Source — standalone ไม่ผ่าน SPA router
// ใช้: /overlay.html?token=<JWT> — แสดงเฉพาะโดเนตของสตรีมเมอร์ที่ถือ token นั้น

import { api, getToken, type DonationEvent, type Settings } from '../api/auth'

const params = new URLSearchParams(location.search)
const token = params.get('token') || getToken() || ''

// ---------- DOM ----------
const root = document.getElementById('app')!
root.innerHTML = `
  <div id="alert" style="
    position:fixed; top:44%; left:50%; transform:translate(-50%,-50%) scale(0) rotate(-2deg);
    width:560px; padding:26px 30px 22px; border-radius:22px; text-align:left;
    background:linear-gradient(160deg, rgba(17,26,45,.94), rgba(11,17,32,.96));
    border:1.5px solid rgba(148,163,184,.28);
    box-shadow:0 24px 70px rgba(0,0,0,.6), 0 0 70px rgba(244,63,94,.28);
    opacity:0; transition:transform .55s cubic-bezier(.2,1.6,.4,1), opacity .3s;
    font-family:'Noto Sans Thai','Segoe UI',sans-serif; color:#e8eef9;">
    <div style="position:absolute; inset:0; border-radius:22px; overflow:hidden; pointer-events:none; border-radius:22px;">
      <div id="a-shine" style="position:absolute; top:-60%; left:-80%; width:45%; height:220%;
        background:linear-gradient(100deg, transparent, rgba(255,255,255,.18), transparent);
        transform:rotate(15deg);"></div>
    </div>
    <!-- header row -->
    <div id="a-head" style="display:flex; align-items:center; justify-content:space-between; position:relative;">
      <span id="a-badge" style="display:inline-flex; align-items:center; gap:7px; padding:7px 16px;
        border-radius:999px; font-size:14px; font-weight:800; font-family:'Plus Jakarta Sans',sans-serif;
        background:linear-gradient(90deg,#fb7185,#f43f5e); color:#fff;">
        ❤️ SUPER CHAT DONATION
      </span>
      <span id="a-tts" style="display:none; align-items:center; gap:7px; padding:6px 13px; border-radius:999px;
        font-size:12px; font-weight:700; background:rgba(16,185,129,.15); color:#34d399;">🔊 TH-TTS</span>
    </div>
    <!-- donor row -->
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
            letter-spacing:.5px; font-family:'Plus Jakarta Sans',sans-serif; background:rgba(251,191,36,.15); color:#fbbf24;">GOLD TIER</span>
        </div>
      </div>
    </div>
    <!-- message bubble -->
    <div id="a-msg-wrap" style="margin-top:16px; padding:14px 18px; border-radius:14px;
      background:rgba(148,163,184,.1); border:1px solid rgba(148,163,184,.15); position:relative;">
      <div id="a-msg" style="font-size:16px; line-height:1.6; color:#e2e8f0;"></div>
    </div>
    <!-- footer -->
    <div style="display:flex; align-items:center; justify-content:space-between; margin-top:14px; position:relative;">
      <span id="a-sound" style="font-size:12px; color:#94a3b8;">♪ เสียงประกอบ</span>
      <span style="font-size:12px; color:#f43f5e; font-weight:700;">● Donate Me ❤️ Engine</span>
    </div>
  </div>
  <style>
    @keyframes bounce { from { transform:translateY(0); } to { transform:translateY(-8px); } }
    @keyframes shine { from { left:-80%; } to { left:160%; } }
    .confetti { position:fixed; font-size:22px; pointer-events:none; animation:fall 3s linear forwards; z-index:-1; }
    @keyframes fall { to { transform:translateY(105vh) rotate(720deg); opacity:0; } }
  </style>
`

document.body.style.cssText = 'margin:0;height:100%;background:rgba(0,0,0,0);overflow:hidden;font-family:Segoe UI,Noto Sans Thai,sans-serif'

const alertBox = root.querySelector<HTMLElement>('#alert')!
const elBadge = root.querySelector<HTMLElement>('#a-badge')!
const elTts = root.querySelector<HTMLElement>('#a-tts')!
const elAvatar = root.querySelector<HTMLElement>('#a-avatar')!
const elName = root.querySelector<HTMLElement>('#a-name')!
const elAmount = root.querySelector<HTMLElement>('#a-amount')!
const elTier = root.querySelector<HTMLElement>('#a-tier')!
const elMsg = root.querySelector<HTMLElement>('#a-msg')!
const elSound = root.querySelector<HTMLElement>('#a-sound')!
const elShine = root.querySelector<HTMLElement>('#a-shine')!

function playShine() {
  elShine.style.animation = 'none'
  void elShine.offsetWidth // restart animation
  elShine.style.animation = 'shine 1.1s ease-out .35s'
}

// tier ตามยอด — สไตล์ SUPER CHAT ตามดีไซน์ Stitch
function tierOf(amount: number): {
  scale: number; badge: string; tier: string; tierColor: string; avatar: string;
  confettiCount: number; soundLabel: string;
} {
  if (amount >= 500) return { scale: 1.22, badge: '❤️ SUPER CHAT DONATION', tier: 'GOLD TIER', tierColor: '#fbbf24', avatar: '👑', confettiCount: 80, soundLabel: '♪ Sound: Level_Up_Super.mp3' }
  if (amount >= 100) return { scale: 1.08, badge: '⭐ VIP DONATION', tier: 'VIP TIER', tierColor: '#a78bfa', avatar: '💜', confettiCount: 50, soundLabel: '♪ Sound: Vip_Glow.mp3' }
  return { scale: 1, badge: '❤️ DONATION', tier: 'SUPPORT TIER', tierColor: '#34d399', avatar: '💗', confettiCount: 30, soundLabel: '♪ Sound: Standard_Support.mp3' }
}

let settings: Settings = {
  theme: 'pink',
  goal_amount: 0,
  alert_duration_sec: 6,
  tts_enabled: true,
  alert_text: 'ขอบคุณ {name} ที่โดเนต {amount} บาท',
  alert_sound_url: '',
  alert_image_url: '',
  show_leaderboard: true,
}

async function loadSettings() {
  if (!token) return
  try {
    settings = await api.getSettings()
  } catch {
    /* ใช้ค่า default ถ้าดึงไม่ได้ */
  }
}

function applyTheme() {
  // ธีมสี accent ของ badge ตาม settings.theme (เข้ากับกรอบกระจกมืดตายดีไซน์ Stitch)
  const hues: Record<string, [string, string]> = {
    pink: ['#fb7185', '#f43f5e'],
    blue: ['#60a5fa', '#3b82f6'],
    green: ['#34d399', '#10b981'],
    dark: ['#94a3b8', '#64748b'],
  }
  const [c1, c2] = hues[settings.theme] ?? hues.pink!
  elBadge.style.background = `linear-gradient(90deg, ${c1}, ${c2})`
}

// ---------- เสียงสังเคราะห์ Web Audio (ไม่ต้องมีไฟล์) ----------
let audioCtx: AudioContext | undefined
function ctx(): AudioContext {
  audioCtx ??= new AudioContext()
  return audioCtx
}

// browser บล็อก autoplay: ถ้า AudioContext ถูก suspend ให้โชว์ปุ่มกดเปิดเสียง
function showUnmuteIfNeeded() {
  if (ctx().state !== 'suspended' || document.getElementById('unmute-chip')) return
  const chip = document.createElement('button')
  chip.id = 'unmute-chip'
  chip.textContent = '🔇 กดเพื่อเปิดเสียงแจ้งเตือน'
  chip.style.cssText =
    'position:fixed;bottom:18px;left:50%;transform:translateX(-50%);z-index:999;' +
    'padding:10px 20px;border:none;border-radius:999px;cursor:pointer;font-size:14px;font-weight:700;' +
    "background:#f43f5e;color:#fff;font-family:'Noto Sans Thai',sans-serif;box-shadow:0 6px 20px rgba(244,63,94,.5)"
  chip.onclick = () => {
    void ctx().resume()
    chip.remove()
  }
  document.body.appendChild(chip)
  // พยายาม resume เองเมื่อผู้ใช้คลิกที่ใดก็ได้ในหน้า
  document.addEventListener('click', () => void ctx().resume(), { once: true })
}
showUnmuteIfNeeded()
ctx().addEventListener?.('statechange', showUnmuteIfNeeded)

function tone(freq: number, start: number, dur: number, type: OscillatorType = 'sine', vol = 0.3) {
  const c = ctx()
  const o = c.createOscillator()
  const g = c.createGain()
  o.type = type
  o.frequency.value = freq
  g.gain.setValueAtTime(vol, c.currentTime + start)
  g.gain.exponentialRampToValueAtTime(0.001, c.currentTime + start + dur)
  o.connect(g).connect(c.destination)
  o.start(c.currentTime + start)
  o.stop(c.currentTime + start + dur)
}

const sounds: Record<string, () => void> = {
  chime: () => [880, 1108.7, 1318.5, 1760].forEach((f, i) => tone(f, i * 0.12, 0.8, 'sine', 0.25)),
  coin: () => {
    tone(988, 0, 0.09, 'square', 0.2)
    tone(1319, 0.09, 0.5, 'square', 0.2)
  },
  fanfare: () => {
    ;[
      [523, 0], [523, 0.15], [523, 0.3], [659, 0.45], [784, 0.65],
      [659, 0.85], [784, 1.0], [1047, 1.15],
    ].forEach(([f, t]) => tone(f, t, 0.35, 'sawtooth', 0.12))
  },
}

function speak(d: DonationEvent) {
  // ข้อความ render เป็น text ล้วน — ปลอดภัยจาก XSS
  const text = settings.alert_text
    .replaceAll('{name}', d.donor_name)
    .replaceAll('{amount}', d.amount.toLocaleString())
  const u = new SpeechSynthesisUtterance(`${text}${d.message ? `. ${d.message}` : ''}`)
  u.lang = 'th-TH'
  // เครื่องไม่มี voice เลย (เช่น Windows ไม่ได้ติดตั้งภาษาไทย) → TTS เงียบ
  // fallback: เล่นเสียงแตรวงแทน พร้อมข้อความแจ้งใน console ของ OBS
  const voices = speechSynthesis.getVoices()
  if (voices.length === 0) {
    console.warn('[Donate Me] ไม่พบ voice TTS ในเครื่อง — เล่นเสียงแตรวงแทน (ติดตั้งภาษาไทยใน OS เพื่อใช้ TTS)')
    sounds.fanfare()
    return
  }
  const thVoice = voices.find((v) => v.lang.startsWith('th'))
  if (thVoice) u.voice = thVoice
  speechSynthesis.cancel()
  speechSynthesis.speak(u)
}

// ---------- ป็อบอัพ + คอนเฟตติ ----------
let hideTimer: ReturnType<typeof setTimeout> | undefined

function showAlert(d: DonationEvent) {
  clearTimeout(hideTimer)
  // กัน XSS: ใช้ textContent เท่านั้น ห้าม innerHTML กับข้อมูลจากผู้ใช้
  const tier = tierOf(d.amount)
  elName.textContent = d.donor_name
  elAmount.textContent = `฿${d.amount.toLocaleString()}`
  elAvatar.textContent = tier.avatar
  elMsg.textContent = d.message
  elBadge.textContent = tier.badge
  elTier.textContent = tier.tier
  elTier.style.color = tier.tierColor
  elTier.style.background = `${tier.tierColor}26`
  elSound.textContent = tier.soundLabel
  elTts.style.display = d.sound === 'tts' ? 'inline-flex' : 'none'
  // tier เอฟเฟกต์: ยอดสูง = ป็อบอัพใหญ่ขึ้น + คอนเฟตติเยอะขึ้น
  alertBox.style.transform = `translate(-50%,-50%) scale(0) rotate(-2deg)`
  applyTheme()
  requestAnimationFrame(() => {
    alertBox.style.transform = `translate(-50%,-50%) scale(${tier.scale}) rotate(0deg)`
    alertBox.style.opacity = '1'
  })
  playShine()
  confetti(tier.confettiCount)

  if (d.sound === 'tts' && settings.tts_enabled) {
    speak(d)
  } else if (settings.alert_sound_url) {
    // เสียงอัปโหลดของสตรีมเมอร์ — เล่นทับเสียงสังเคราะห์
    const audio = new Audio(settings.alert_sound_url)
    void audio.play().catch(() => (sounds[d.sound] ?? sounds.chime)())
  } else {
    ;(sounds[d.sound] ?? sounds.chime)()
  }

  hideTimer = setTimeout(() => {
    alertBox.style.transform = 'translate(-50%,-50%) scale(0) rotate(-3deg)'
    alertBox.style.opacity = '0'
  }, settings.alert_duration_sec * 1000)
}

function confetti(count = 30) {
  const emojis = ['🎉', '💜', '✨', '🎊', '💰']
  for (let i = 0; i < count; i++) {
    const el = document.createElement('div')
    el.className = 'confetti'
    el.textContent = emojis[(Math.random() * emojis.length) | 0]!
    el.style.left = `${Math.random() * 100}vw`
    el.style.top = '-30px'
    el.style.animationDelay = `${Math.random() * 0.8}s`
    el.style.animationDuration = `${2.2 + Math.random() * 1.5}s`
    document.body.appendChild(el)
    setTimeout(() => el.remove(), 4500)
  }
}

// ---------- SSE + auto-reconnect ----------
function connect() {
  const url = token ? `/events?token=${encodeURIComponent(token)}` : '/events'
  const es = new EventSource(url)
  es.addEventListener('donation', (e) => {
    showAlert(JSON.parse((e as MessageEvent).data) as DonationEvent)
  })
  es.onerror = () => {
    es.close()
    setTimeout(connect, 3000)
  }
}

void loadSettings().then(connect)

// ทดสอบจาก OBS ได้: เรียก window.testAlert() ใน console ของ Browser Source
declare global {
  interface Window {
    testAlert: () => void
  }
}
window.testAlert = () =>
  showAlert({ user_id: '', id: 'test', donor_name: 'คุณทดสอบ', amount: 100, message: 'นี่คือการทดสอบป็อบอัพ', sound: 'chime' })
