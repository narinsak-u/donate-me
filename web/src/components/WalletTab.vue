<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { dashApi, type Wallet, type Withdrawal } from '../api/dashboard'

const wallet = ref<Wallet | null>(null)
const withdrawals = ref<Withdrawal[]>([])
const amount = ref<number | null>(null)
const bank = ref('')
const account = ref('')
const busy = ref(false)
const error = ref('')

onMounted(load)

async function load() {
  const [w, ws] = await Promise.all([dashApi.wallet(), dashApi.withdrawals()])
  wallet.value = w
  withdrawals.value = ws
}

async function submit() {
  error.value = ''
  busy.value = true
  try {
    await dashApi.requestWithdrawal({
      amount: amount.value ?? 0,
      bank_name: bank.value,
      bank_account: account.value,
    })
    amount.value = null
    bank.value = ''
    account.value = ''
    await load()
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'ส่งคำขอไม่สำเร็จ'
  } finally {
    busy.value = false
  }
}

function fmtDate(ms: number | null): string {
  if (!ms) return '-'
  return new Date(ms).toLocaleString('th-TH', { dateStyle: 'short', timeStyle: 'short' })
}
</script>

<template>
  <div class="wallet">
    <div class="metrics">
      <div class="metric">
        <div class="metric-head"><span>ยอดถอนได้ (BALANCE)</span><i class="m-icon green">💎</i></div>
        <b class="metric-num">฿{{ (wallet?.balance ?? 0).toLocaleString() }}</b>
        <span class="metric-foot">พร้อมถอนทันที</span>
      </div>
      <div class="metric">
        <div class="metric-head"><span>รอดำเนินการ</span><i class="m-icon gold">⏳</i></div>
        <b class="metric-num">฿{{ (wallet?.pending_withdraw ?? 0).toLocaleString() }}</b>
        <span class="metric-foot">คำขอถอนที่รอโอน</span>
      </div>
      <div class="metric">
        <div class="metric-head"><span>ถอนสำเร็จแล้ว</span><i class="m-icon pink">🏦</i></div>
        <b class="metric-num">฿{{ (wallet?.total_withdrawn ?? 0).toLocaleString() }}</b>
        <span class="metric-foot">ตลอดการใช้งาน</span>
      </div>
      <div class="metric">
        <div class="metric-head"><span>รายได้รวม</span><i class="m-icon blue">📈</i></div>
        <b class="metric-num">฿{{ (wallet?.total_earned ?? 0).toLocaleString() }}</b>
        <span class="metric-foot">จากโดเนตทั้งหมด</span>
      </div>
    </div>

    <section class="card">
      <div class="card-head">
        <h2>🏦 ขอถอนเงิน</h2>
        <span class="badge badge-gold">MOCK — ยังไม่มีการโอนจริง</span>
      </div>
      <p class="muted" style="margin-bottom: 6px">
        ระบบจำลองการถอน (สถานะ "รอดำเนินการ") — การโอนเข้าบัญชีจริงจะเปิดใช้เมื่อต่อ Omise Payout ในอนาคต
      </p>
      <div class="withdraw-form">
        <div>
          <label>จำนวนเงิน (ขั้นต่ำ ฿100)</label>
          <input v-model.number="amount" class="input" type="number" min="100" :max="wallet?.balance" placeholder="เช่น 500" />
        </div>
        <div>
          <label>ธนาคาร</label>
          <input v-model="bank" class="input" placeholder="เช่น กสิกรไทย (KBank)" maxlength="60" />
        </div>
        <div>
          <label>เลขบัญชี</label>
          <input v-model="account" class="input" placeholder="เช่น 1234567890" maxlength="30" />
        </div>
        <button class="btn-primary" :disabled="busy || !amount || (wallet?.balance ?? 0) < 100" @click="submit">
          {{ busy ? 'กำลังส่ง...' : '💸 ส่งคำขอถอน' }}
        </button>
      </div>
      <p v-if="error" class="error">{{ error }}</p>
    </section>

    <section class="card">
      <div class="card-head">
        <h2>📋 ประวัติการถอน</h2>
      </div>
      <table v-if="withdrawals.length">
        <thead>
          <tr><th>เวลา</th><th>ยอดเงิน</th><th>ธนาคาร</th><th>เลขบัญชี</th><th>สถานะ</th></tr>
        </thead>
        <tbody>
          <tr v-for="w in withdrawals" :key="w.id">
            <td class="muted">{{ fmtDate(w.created_at) }}</td>
            <td class="amount">฿{{ w.amount.toLocaleString() }}</td>
            <td>{{ w.bank_name }}</td>
            <td class="muted">{{ w.bank_account }}</td>
            <td>
              <span class="status-pill" :class="w.status">
                {{ w.status === 'pending' ? '⏳ รอดำเนินการ' : w.status === 'completed' ? '✅ สำเร็จ' : '❌ ถูกปฏิเสธ' }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-else class="muted center" style="padding: 20px">ยังไม่มีประวัติการถอน</p>
    </section>
  </div>
</template>

<style scoped>
.wallet { display: grid; gap: 18px; }
.metrics { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; }
.metric {
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius-lg); padding: 18px; box-shadow: var(--shadow-card);
}
.metric-head { display: flex; justify-content: space-between; align-items: center; font-size: 11.5px; color: var(--text-faint); font-weight: 700; }
.m-icon { width: 34px; height: 34px; border-radius: 11px; font-style: normal; display: flex; align-items: center; justify-content: center; font-size: 15px; }
.m-icon.pink { background: var(--primary-soft); }
.m-icon.green { background: var(--emerald-soft); }
.m-icon.gold { background: rgba(251, 191, 36, 0.14); }
.m-icon.blue { background: rgba(59, 130, 246, 0.14); }
.metric-num { display: block; font-family: var(--font-head); font-size: 29px; font-weight: 800; margin: 12px 0 6px; font-variant-numeric: tabular-nums; }
.metric-foot { font-size: 11.5px; color: var(--text-faint); }
.card {
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: var(--radius-xl); padding: 22px; box-shadow: var(--shadow-card);
}
.card-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; }
.card-head h2 { font-size: 16px; }
.muted { color: var(--text-dim); font-size: 12.5px; }
label { display: block; font-size: 13px; font-weight: 600; color: var(--text-dim); margin: 12px 0 7px; }
.input {
  width: 100%; padding: 12px 14px; border-radius: var(--radius-md);
  border: 1px solid var(--border); background: var(--bg-input);
  color: var(--text); font-size: 14.5px; font-family: var(--font-body); outline: none;
}
.input:focus { border-color: var(--primary); box-shadow: 0 0 0 3px var(--primary-soft); }
.withdraw-form { display: grid; grid-template-columns: 2fr 1fr 1fr auto; gap: 12px; align-items: end; margin-top: 8px; }
.btn-primary {
  padding: 13px 22px; border: none; border-radius: var(--radius-md);
  background: linear-gradient(135deg, #fb7185, var(--primary)); color: #fff;
  font-size: 15px; font-weight: 700; font-family: var(--font-head); cursor: pointer;
}
.btn-primary:disabled { opacity: 0.55; cursor: default; }
.error { color: var(--primary); font-size: 13px; margin-top: 12px; }
table { width: 100%; border-collapse: collapse; font-size: 13.5px; }
th { text-align: left; color: var(--text-faint); font-weight: 700; padding: 9px 8px; border-bottom: 1px solid var(--border); }
td { padding: 10px 8px; border-bottom: 1px solid var(--hover-row); }
td.amount { color: var(--gold); font-weight: 700; font-family: var(--font-head); }
.center { text-align: center; }
.status-pill { font-size: 12px; font-weight: 700; }
.status-pill.pending { color: var(--gold); }
.status-pill.completed { color: var(--emerald); }
.status-pill.rejected { color: var(--primary); }
@media (max-width: 900px) {
  .metrics { grid-template-columns: 1fr 1fr; }
  .withdraw-form { grid-template-columns: 1fr; }
}
</style>
