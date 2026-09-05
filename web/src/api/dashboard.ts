// API ฝั่ง dashboard (Phase 3) — ตรงกับ server/src/dashboard.rs

import { request } from './auth'

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
}
