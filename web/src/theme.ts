// ธีมกลาง — ใช้ร่วมทุกหน้า (แก้ที่เดียว)

const THEME_KEY = 'donateme_theme'

export function currentTheme(): 'dark' | 'light' {
  return (localStorage.getItem(THEME_KEY) as 'dark' | 'light') ?? 'dark'
}

export function applyTheme(theme?: 'dark' | 'light'): 'dark' | 'light' {
  const t = theme ?? currentTheme()
  localStorage.setItem(THEME_KEY, t)
  document.documentElement.setAttribute('data-theme', t)
  return t
}

export function toggleTheme(): 'dark' | 'light' {
  return applyTheme(currentTheme() === 'dark' ? 'light' : 'dark')
}
