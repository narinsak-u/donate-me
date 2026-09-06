// API ฝั่ง dashboard (Phase 3) — ตรงกับ server/src/dashboard.rs

import { getToken, request } from './auth'

export interface DayPoint {
  day: string
  total: number
}

export interface Stats {
  total_all: number
  total_today: number
  total_month: number
  count_all: number
  goal_amount: number
  series_30d: DayPoint[]
}

export interface DonationItem {
  id: string
  donor_name: string
  amount: number
  message: string
  sound: string
  hidden: boolean
  pinned: boolean
  created_at: number
  paid_at: number | null
}

export interface HistoryResponse {
  items: DonationItem[]
  page: number
  total_pages: number
}

export interface Wallet {
  total_earned: number
  pending_withdraw: number
  total_withdrawn: number
  balance: number
}

export interface Withdrawal {
  id: string
  amount: number
  bank_name: string
  bank_account: string
  status: 'pending' | 'completed' | 'rejected'
  created_at: number
  paid_at: number | null
}

// ---------- Phase 7: คิวสลิป (โอนตรง) ----------

export interface SlipItem {
  id: string
  donor_name: string
  amount: number
  message: string
  sound: string
  status: 'awaiting_review' | 'paid' | 'rejected' | string
  has_image: boolean
  created_at: number
  slip_at: number | null
  reviewed_at: number | null
  review_note: string
}

export const dashApi = {
  stats: () => request<Stats>('/api/me/stats'),
  history: (q?: { query?: string; page?: number; from?: number; to?: number }) => {
    const params = new URLSearchParams()
    if (q?.query) params.set('q', q.query)
    if (q?.page) params.set('page', String(q.page))
    if (q?.from) params.set('from', String(q.from))
    if (q?.to) params.set('to', String(q.to))
    return request<HistoryResponse>(`/api/me/donations?${params}`)
  },
  setHidden: (id: string, hidden: boolean) =>
    request<{ ok: boolean }>(`/api/me/donations/${id}`, {
      method: 'PATCH',
      body: JSON.stringify({ hidden }),
    }),
  setPinned: (id: string, pinned: boolean) =>
    request<{ ok: boolean }>(`/api/me/donations/${id}`, {
      method: 'PATCH',
      body: JSON.stringify({ pinned }),
    }),
  csvUrl: (from?: number, to?: number) => {
    const params = new URLSearchParams()
    if (from) params.set('from', String(from))
    if (to) params.set('to', String(to))
    const qs = params.toString()
    return `/api/me/donations.csv${qs ? '?' + qs : ''}`
  },
  uploadSound: async (file: File) => {
    const fd = new FormData()
    fd.append('file', file)
    const res = await fetch('/api/me/alert-sound', {
      method: 'POST',
      headers: { Authorization: `Bearer ${localStorage.getItem('donateme_token') ?? ''}` },
      body: fd,
    })
    if (!res.ok) throw new Error(await res.text())
    return res.json() as Promise<{ url: string }>
  },
  wallet: () => request<Wallet>('/api/me/wallet'),
  withdrawals: () => request<Withdrawal[]>('/api/me/withdrawals'),
  requestWithdrawal: (body: { amount: number; bank_name: string; bank_account: string }) =>
    request<Withdrawal>('/api/me/withdrawals', { method: 'POST', body: JSON.stringify(body) }),
  // คิวสลิป — อนุมัติแล้ว alert เด้งทันที, ปฏิเสธแนบเหตุผลให้ผู้ชมเห็น
  slips: () => request<SlipItem[]>('/api/me/slips'),
  /// <img> ส่ง header ไม่ได้ → ใช้ ?token= (extractor รองรับ)
  slipImageUrl: (id: string) => `/api/me/slips/${id}/image?token=${encodeURIComponent(getToken() ?? '')}`,
  approveSlip: (id: string) =>
    request<{ ok: boolean; status: string }>(`/api/me/slips/${id}/approve`, { method: 'POST' }),
  rejectSlip: (id: string, note: string) =>
    request<{ ok: boolean; status: string }>(`/api/me/slips/${id}/reject`, {
      method: 'POST',
      body: JSON.stringify({ note }),
    }),
  /// QR ทดสอบพร้อมเพย์ของตัวเอง — เป็น route ที่ต้อง auth จึงแนบ token ใน query
  promptpayQrUrl: (amount: number) =>
    `/api/me/promptpay-qr?amount=${amount}&token=${encodeURIComponent(getToken() ?? '')}`,
}
