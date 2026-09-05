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
