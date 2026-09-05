import { onUnmounted, ref } from 'vue'
import { api, type SoundKind } from '../api/auth'
import { playSound, primeTts, speakThai } from '../sounds'
import { burstConfetti } from '../confetti'

// ดูสถานะ Donation — ใช้ร่วมกันระหว่างแผง QR บนหน้าโดเนตกับหน้า /pay/:id (ADR-0001)
// เรียก start() หนึ่งครั้งต่อ donation, หยุดเองเมื่อ status ออกจาก pending
export type DonationStatus = 'pending' | 'paid' | 'failed' | 'expired'

// ข้อความเริ่มต้นที่จะถูกอ่าน กรณีผู้บริจาคไม่พิมพ์ข้อความ
export const DEFAULT_DONOR_MSG = 'เป็นกำลังใจให้นะ'

// อ่านข้อความผู้บริจาคทุกครั้ง หลังเสียงที่เลือก (ตัดสั้น ๆ กันยาวเกิน)
function donorTtsText(name: string, amount: number, message?: string): string {
  const msg = message?.trim() || DEFAULT_DONOR_MSG
  return `ขอบคุณ ${name} ที่โดเนต ${amount.toLocaleString()} บาท. ${msg.slice(0, 120)}`
}

export function useDonationWatch() {
  const status = ref<DonationStatus | null>(null)
  const secondsLeft = ref(15 * 60)
  let timer: ReturnType<typeof setInterval> | undefined

  const countdown = ref('15:00')

  // เฉลิมฉลองเมื่อจ่ายสำเร็จ — เสียงที่เลือกนำ แล้วอ่านข้อความตามเสมอ
  function celebrate(sound: SoundKind, opts: { name: string; amount: number; message?: string }) {
    playSound(sound)
    setTimeout(() => void speakThai(donorTtsText(opts.name, opts.amount, opts.message)), 1400)
    burstConfetti(80)
  }

  function stop() {
    if (timer) clearInterval(timer)
    timer = undefined
  }

  function start(
    id: string,
    opts: { sound: SoundKind; name: string; amount: number; message?: string },
  ) {
    stop()
    primeTts() // อุ่นรายชื่อ voice ล่วงหน้า กันโหมด tts พูดไม่ออก
    status.value = 'pending'
    secondsLeft.value = 15 * 60
    countdown.value = '15:00'
    timer = setInterval(async () => {
      secondsLeft.value = Math.max(0, secondsLeft.value - 1)
      const m = Math.floor(secondsLeft.value / 60)
      const s = secondsLeft.value % 60
      countdown.value = `${m}:${String(s).padStart(2, '0')}`
      try {
        const res = await api.getDonation(id)
        status.value = res.status
        if (res.status !== 'pending') {
          stop()
          if (res.status === 'paid') celebrate(opts.sound, opts)
        }
      } catch {
        /* network สะดุดชั่วคราว — รอรอบถัดไป */
      }
    }, 1000)
  }

  onUnmounted(stop)
  return { status, countdown, start, stop }
}
