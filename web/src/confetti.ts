// คอนเฟตติใช้ได้ทุกหน้า — import { burstConfetti } หรือเรียก window.confetti() จาก console

const DEFAULT_EMOJIS = ['🎉', '💜', '✨', '🎊', '💰', '❤️']

export function burstConfetti(
  count = 60,
  emojis: string[] = DEFAULT_EMOJIS,
  opts?: { behind?: boolean },
) {
  const zindex = opts?.behind ? '-1' : '9999'
  for (let i = 0; i < count; i++) {
    const el = document.createElement('div')
    el.className = 'dm-confetti'
    el.textContent = emojis[(Math.random() * emojis.length) | 0]!
    el.style.left = `${Math.random() * 100}vw`
    el.style.top = '-30px'
    el.style.fontSize = `${16 + Math.random() * 14}px`
    el.style.animationDelay = `${Math.random() * 0.8}s`
    el.style.animationDuration = `${2.2 + Math.random() * 1.5}s`
    el.style.zIndex = zindex
    document.body.appendChild(el)
    setTimeout(() => el.remove(), 4500)
  }

  // สไตล์ฉีดครั้งเดียว
  if (!document.getElementById('dm-confetti-style')) {
    const style = document.createElement('style')
    style.id = 'dm-confetti-style'
    style.textContent = `
      .dm-confetti {
        position: fixed; top: -30px; z-index: 9999; pointer-events: none;
        animation: dm-confetti-fall 3s linear forwards;
      }
      @keyframes dm-confetti-fall {
        to { transform: translateY(105vh) rotate(720deg); opacity: 0; }
      }
    `
    document.head.appendChild(style)
  }
}

declare global {
  interface Window {
    confetti: typeof burstConfetti
  }
}
