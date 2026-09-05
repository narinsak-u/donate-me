// เสียงสังเคราะห์ Web Audio ใช้ร่วมกันระหว่าง overlay และหน้า pay (ไม่ต้องมีไฟล์เสียง)

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

export const sounds: Record<string, () => void> = {
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

/** เล่นเสียงตามชนิดที่เลือก — resume ก่อนเสมอ (กัน autoplay policy) */
export function playSound(kind: string) {
  void ctx().resume().catch(() => {})
  setTimeout(() => (sounds[kind] ?? sounds.chime)(), 60)
}

/** อุ่นเครื่อง TTS — เรียกตอนเริ่ม watch เพื่อให้ Chrome โหลดรายชื่อ voice ล่วงหน้า */
export function primeTts() {
  if (!('speechSynthesis' in window)) return
  speechSynthesis.getVoices() // ยิงครั้งแรกเพื่อ trigger voiceschanged
}

/** รอรายชื่อ voice — Chrome จะคืน [] จนกว่า voiceschanged จะยิง จึงต้องรอแบบมี timeout */
function loadVoices(): Promise<SpeechSynthesisVoice[]> {
  const first = speechSynthesis.getVoices()
  if (first.length > 0) return Promise.resolve(first)
  return new Promise((resolve) => {
    let settled = false
    const done = (v: SpeechSynthesisVoice[]) => {
      if (settled) return
      settled = true
      speechSynthesis.removeEventListener('voiceschanged', onChange)
      clearInterval(poll)
      resolve(v)
    }
    const check = () => {
      const v = speechSynthesis.getVoices()
      if (v.length > 0) done(v)
    }
    const onChange = () => check()
    const poll = setInterval(check, 300)
    speechSynthesis.addEventListener('voiceschanged', onChange)
    setTimeout(() => done(speechSynthesis.getVoices()), 2500)
  })
}

/** อ่านข้อความไทยด้วย TTS — ไม่มี voice ในเครื่อง → เล่นแตรวงแทน */
export async function speakThai(text: string, speed = 1.0) {
  if (!('speechSynthesis' in window)) {
    sounds.fanfare()
    return
  }
  const voices = await loadVoices()
  if (voices.length === 0) {
    console.warn('[Donate Me] ไม่พบ voice TTS ในเครื่อง — เล่นเสียงแตรวงแทน (ติดตั้งภาษาไทยใน OS เพื่อใช้ TTS)')
    sounds.fanfare()
    return
  }
  const u = new SpeechSynthesisUtterance(text)
  u.lang = 'th-TH'
  u.rate = speed
  const th = voices.find((v) => v.lang.startsWith('th'))
  if (th) u.voice = th
  speechSynthesis.cancel()
  speechSynthesis.speak(u)
}

/** ลำดับป็อบอัพโหมด TTS: กระดิ่งนำ → เสียงอ่านตามหลัง 1.4 วิ */
export function playTtsSequence(text: string, speed = 1.0, voiceDelayMs = 1400) {
  playSound('chime')
  setTimeout(() => speakThai(text, speed), voiceDelayMs)
}
