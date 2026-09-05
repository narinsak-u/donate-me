// Overlay สำหรับ OBS Browser Source — standalone ไม่ผ่าน SPA router
// ใช้: /overlay.html?token=<JWT> — แสดงเฉพาะโดเนตของสตรีมเมอร์ที่ถือ token นั้น

import { api, getToken, type DonationEvent, type Settings } from '../api/auth'

const params = new URLSearchParams(location.search)
const token = params.get('token') || getToken() || ''

// ---------- DOM ----------
const root = document.getElementById('app')!
root.innerHTML = `
  <div id="alert" style="
    position:fixed; top:40%; left:50%; transform:translate(-50%,-50%) scale(0);
    padding:28px 48px; border-radius:20px; text-align:center;
    box-shadow:0 0 60px rgba(255,110,199,.6), 0 20px 40px rgba(0,0,0,.5);
    border:3px solid rgba(255,255,255,.5); opacity:0;
    transition:transform .5s cubic-bezier(.2,1.6,.4,1), opacity .3s;">
    <div id="a-icon" style="font-size:52px; animation:bounce 1s infinite alternate;">❤️</div>
    <img id="a-img" alt="" style="display:none; max-width:260px; max-height:160px; border-radius:12px; margin-top:10px;">
    <div id="a-name" style="font-size:26px; font-weight:800; color:#fff; margin-top:8px; text-shadow:0 2px 8px rgba(0,0,0,.4);"></div>
    <div id="a-amount" style="font-size:40px; font-weight:900; color:#ffe066; text-shadow:0 2px 10px rgba(0,0,0,.5); margin-top:2px;"></div>
    <div id="a-msg" style="font-size:17px; color:#fff; margin-top:10px; max-width:420px; text-shadow:0 1px 4px rgba(0,0,0,.4);"></div>
  </div>
  <style>
    @keyframes bounce { from { transform:translateY(0); } to { transform:translateY(-10px); } }
    .confetti { position:fixed; font-size:22px; pointer-events:none; animation:fall 3s linear forwards; }
    @keyframes fall { to { transform:translateY(105vh) rotate(720deg); opacity:0; } }
  </style>
`

document.body.style.cssText = 'margin:0;height:100%;background:rgba(0,0,0,0);overflow:hidden;font-family:Segoe UI,Noto Sans Thai,sans-serif'

const alertBox = root.querySelector<HTMLElement>('#alert')!
const elIcon = root.querySelector<HTMLElement>('#a-icon')!
const elImg = root.querySelector<HTMLImageElement>('#a-img')!
const elName = root.querySelector<HTMLElement>('#a-name')!
const elAmount = root.querySelector<HTMLElement>('#a-amount')!
const elMsg = root.querySelector<HTMLElement>('#a-msg')!

const THEMES: Record<string, { bg: string; glow: string; icon: string }> = {
  pink: { bg: 'linear-gradient(135deg, rgba(255,110,199,.95), rgba(120,115,245,.95))', glow: 'rgba(255,110,199,.6)', icon: '❤️' },
  blue: { bg: 'linear-gradient(135deg, rgba(56,132,255,.95), rgba(26,42,108,.95))', glow: 'rgba(56,132,255,.6)', icon: '💙' },
  green: { bg: 'linear-gradient(135deg, rgba(34,197,94,.95), rgba(15,90,60,.95))', glow: 'rgba(34,197,94,.6)', icon: '💚' },
  dark: { bg: 'linear-gradient(135deg, rgba(30,30,40,.97), rgba(10,10,20,.97))', glow: 'rgba(255,255,255,.15)', icon: '🖤' },
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

// tier เอฟเฟกต์ตามระดับยอดโดเนต
function tierOf(amount: number): { scale: number; emoji: string; confettiCount: number; extra: string } {
  if (amount >= 500) return { scale: 1.35, emoji: '🤩', confettiCount: 80, extra: 'SUPER CHAT!' }
  if (amount >= 100) return { scale: 1.15, emoji: '🎉', confettiCount: 50, extra: '' }
  return { scale: 1, emoji: '', confettiCount: 30, extra: '' }
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
  const t = THEMES[settings.theme] ?? THEMES.pink!
  alertBox.style.background = t.bg
  alertBox.style.boxShadow = `0 0 60px ${t.glow}, 0 20px 40px rgba(0,0,0,.5)`
  elIcon.textContent = t.icon
}

// ---------- เสียงสังเคราะห์ Web Audio (ไม่ต้องมีไฟล์) ----------
let audioCtx: AudioContext | undefined
function ctx(): AudioContext {
  audioCtx ??= new AudioContext()
  return audioCtx
}

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
  elMsg.textContent = d.message
  // media alert: รูป/GIF จาก settings (เฉพาะ https URL ที่ตรวจแล้วฝั่ง server)
  if (settings.alert_image_url) {
    elImg.src = settings.alert_image_url
    elImg.style.display = 'inline-block'
  } else {
    elImg.style.display = 'none'
  }
  // tier เอฟเฟกต์: ยอดสูง = ป็อบอัพใหญ่ขึ้น + คอนเฟตติเยอะขึ้น
  alertBox.style.transform = `translate(-50%,-50%) scale(0)`
  applyTheme()
  if (tier.emoji) elIcon.textContent = tier.emoji
  elIcon.style.animationDuration = tier.scale >= 1.35 ? '0.4s' : '1s'
  requestAnimationFrame(() => {
    alertBox.style.transform = `translate(-50%,-50%) scale(${tier.scale})`
    alertBox.style.opacity = '1'
  })
  confetti(tier.confettiCount)

  if (tier.extra) {
    elMsg.textContent = `${tier.extra} ${d.message}`.trim()
  }

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
    alertBox.style.transform = 'translate(-50%,-50%) scale(0)'
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
