export type SoundKind = 'chime' | 'coin' | 'fanfare' | 'tts'

// ---------- auth ----------

export interface PublicUser {
  id: string
  username: string
  display_name: string
}

export interface AuthResponse {
  token: string
  user: PublicUser
}

const TOKEN_KEY = 'donateme_token'
const USER_KEY = 'donateme_user'

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY)
}

export function getUser(): PublicUser | null {
  try {
    const raw = localStorage.getItem(USER_KEY)
    return raw ? (JSON.parse(raw) as PublicUser) : null
  } catch {
    return null
  }
}

export function saveSession(token: string, user: PublicUser): void {
  localStorage.setItem(TOKEN_KEY, token)
  localStorage.setItem(USER_KEY, JSON.stringify(user))
}

export function clearSession(): void {
  localStorage.removeItem(TOKEN_KEY)
  localStorage.removeItem(USER_KEY)
}

// ---------- api client (มี token อัตโนมัติ) ----------

export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  const token = getToken()
  if (token) headers.Authorization = `Bearer ${token}`

  const res = await fetch(path, { ...init, headers })
  if (!res.ok) {
    const err = new Error(await res.text())
    ;(err as Error & { status?: number }).status = res.status
    throw err
  }
  return res.json() as Promise<T>
}

export const authApi = {
  async register(body: { username: string; email: string; password: string; display_name?: string }) {
    const res = await request<AuthResponse>('/api/auth/register', { method: 'POST', body: JSON.stringify(body) })
    saveSession(res.token, res.user)
    return res
  },
  async login(body: { email: string; password: string }) {
    const res = await request<AuthResponse>('/api/auth/login', { method: 'POST', body: JSON.stringify(body) })
    saveSession(res.token, res.user)
    return res
  },
  me: () => request<PublicUser>('/api/me'),
}

// ---------- donations + settings ----------

export interface CreateDonationInput {
  name: string
  amount: number
  message?: string
  sound?: SoundKind
  username?: string
}

export interface DonationCreated {
  id: string
  qr_url: string
  pay_url: string
  /// ช่องทางจ่าย: mock (จำลอง) | direct (โอนตรง + แนบสลิป) | omise (เกตเวย์)
  mode: 'mock' | 'direct' | 'omise' | string
}

export interface DonationStatusResponse {
  id: string
  status: 'pending' | 'awaiting_review' | 'paid' | 'failed' | 'expired' | 'rejected'
  amount: number
  mode: 'mock' | 'direct' | 'omise' | string
  review_note: string
}

export interface SocialLink {
  kind: 'facebook' | 'youtube' | 'twitch' | 'tiktok' | 'x' | string
  url: string
}

export interface PublicProfile {
  username: string
  display_name: string
  goal_amount: number
  goal_raised: number
  theme: string
  show_leaderboard: boolean
  /// รับโอนตรง + แนบสลิป (สตรีมเมอร์ตั้งเบอร์พร้อมเพย์แล้ว)
  accepts_slip: boolean
  // Phase 9: การปรับแต่งหน้า
  page_theme: 'rose' | 'mint' | 'midnight' | 'retro' | string
  cover_url: string
  about_text: string
  socials: SocialLink[]
}

export interface StreamerSummary {
  username: string
  display_name: string
  goal_amount: number
  goal_raised: number
  donor_count: number
}

export interface Settings {
  theme: string
  goal_amount: number
  alert_duration_sec: number
  tts_enabled: boolean
  alert_text: string
  alert_sound_url: string
  alert_image_url: string
  show_leaderboard: boolean
  alert_position: string
  tts_speed: number
  tts_max_len: number
  tier_vip_amount: number
  tier_gold_amount: number
  // บัญชีรับเงินโอนตรง (promptpay_id ว่าง = ใช้ช่องทาง mock/omise)
  promptpay_id: string
  bank_name: string
  bank_no: string
  // Phase 9: ปรับแต่งหน้าโดเนต
  page_theme: string
  cover_url: string
  about_text: string
  social_facebook: string
  social_youtube: string
  social_twitch: string
  social_tiktok: string
  social_x: string
}

export interface TopDonator {
  donor_name: string
  total: number
  count: number
}

export interface RecentDonation {
  donor_name: string
  amount: number
  message: string
  paid_at: number
}

export interface DonationEvent {
  user_id: string
  id: string
  donor_name: string
  amount: number
  message: string
  sound: SoundKind
}

export const api = {
  createDonation: (body: CreateDonationInput) =>
    request<DonationCreated>('/api/donate', { method: 'POST', body: JSON.stringify(body) }),
  getDonation: (id: string) => request<DonationStatusResponse>(`/api/donate/${id}`),
  testAlert: (amount = 100) =>
    request<{ ok: boolean }>('/api/test-alert', { method: 'POST', body: JSON.stringify({ amount }) }),
  publicProfile: (username: string) => request<PublicProfile>(`/api/u/${username}`),
  streamers: () => request<StreamerSummary[]>('/api/streamers'),
  topDonators: (username: string) => request<TopDonator[]>(`/api/u/${username}/top`),
  recentDonations: (username: string, opts?: { limit?: number; before?: number }) => {
    const p = new URLSearchParams()
    if (opts?.limit) p.set('limit', String(opts.limit))
    if (opts?.before) p.set('before', String(opts.before))
    const qs = p.toString()
    return request<RecentDonation[]>(`/api/u/${username}/recent${qs ? `?${qs}` : ''}`)
  },
  getSettings: () => request<Settings>('/api/me/settings'),
  updateSettings: (body: Partial<Settings>) =>
    request<Settings>('/api/me/settings', { method: 'PUT', body: JSON.stringify(body) }),
  leaderboard: () => request<TopDonator[]>('/api/me/leaderboard'),
}
