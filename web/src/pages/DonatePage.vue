<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api, type PublicProfile, type TopDonator, type RecentDonation, type SoundKind } from '../api/auth'
import { applyTheme } from '../theme'
import { useDonationWatch } from '../composables/useDonationWatch'
import SiteTopbar from '../components/SiteTopbar.vue'
import SlipUpload from '../components/SlipUpload.vue'

const props = defineProps<{ username?: string }>()
const route = useRoute()
const router = useRouter()
const profile = ref<PublicProfile | null>(null)
const topDonators = ref<TopDonator[]>([])
const recent = ref<RecentDonation[]>([])
const loadError = ref('')

// หน้าโดเนตอยู่ที่ /u/:username — ลิงก์เก่าแบบ /?u= ถูก redirect ที่ router guard แล้ว
const targetUsername = computed(
  () => props.username || (route.query.u as string) || undefined,
)

// ---------- ฟอร์ม ----------
const name = ref('')
const amount = ref<number | null>(null)
const message = ref('')
const sound = ref<SoundKind>('chime')
const anonymous = ref(false)
const submitting = ref(false)
const error = ref('')

// ---------- สถานะหน้า: form → pay → paid/failed (in-place swap, ADR-0001) ----------
const stage = ref<'form' | 'pay' | 'paid' | 'failed'>('form')
const current = ref<{ id: string; qrUrl: string; payUrl: string; amount: number; sound: SoundKind; name: string; mode: string } | null>(null)
const { status, countdown, start, stop } = useDonationWatch()
let resetTimer: ReturnType<typeof setTimeout> | undefined

watch(status, (s) => {
  if (s === 'paid') {
    stage.value = 'paid'
    void refreshData() // อัปเดตแถบเป้าหมาย/กระดานด้วยข้อมูลหลังจ่าย
    clearTimeout(resetTimer)
    resetTimer = setTimeout(backToForm, 8000) // เฉลิมฉลองแป๊บเดียวแล้วรีเซ็ตรอโดเนตใหม่
  } else if (s === 'expired' || s === 'failed') {
    stage.value = 'failed'
  }
})

// ดึงข้อมูลสดของสตรีมเมอร์ — เรียกตอน mount และหลังจ่ายสำเร็จ
const PAGE_SIZE = 10
const recentHasMore = ref(false)
const recentLoading = ref(false)

// โหลดกำลังใจล่าสุด — reset=true = หน้าแรก (cursor ใหม่), ไม่งั้นต่อจากรายการสุดท้าย
async function loadRecent(reset = false) {
  if (!targetUsername.value) return
  recentLoading.value = true
  try {
    const before = reset ? undefined : recent.value.at(-1)?.paid_at
    const batch = await api.recentDonations(targetUsername.value, {
      limit: PAGE_SIZE,
      before,
    })
    if (reset) recent.value = batch
    else recent.value.push(...batch)
    recentHasMore.value = batch.length >= PAGE_SIZE
  } catch {
    /* ข้อมูลรอบหน้าจะลองใหม่ */
  } finally {
    recentLoading.value = false
  }
}

async function refreshData() {
  if (!targetUsername.value) return
  try {
    const p = await api.publicProfile(targetUsername.value)
    profile.value = p
    if (p.show_leaderboard) {
      // ข้อมูลจริงเท่านั้น — ไม่มีก็แสดง empty state ตาม ADR-0002
      topDonators.value = await api.topDonators(targetUsername.value).then((r) => r.slice(0, 5))
    }
    await loadRecent(true)
  } catch {
    if (!profile.value) loadError.value = 'ไม่พบสตรีมเมอร์นี้ — ตรวจลิงก์อีกครั้ง'
  }
}

const presets = [
  { v: 20, label: 'กาแฟวาร์ป', emoji: '☕' },
  { v: 50, label: 'เชลบี้', emoji: '⚡' },
  { v: 100, label: 'อัคคีไฟ', emoji: '🔥' },
  { v: 500, label: 'ชุดปรับเกียร์', emoji: '🔧' },
]
const sounds: { value: SoundKind; label: string; color: string }[] = [
  { value: 'chime', label: 'กระดิ่ง', color: '#f43f5e' },
  { value: 'coin', label: 'เหรียญ', color: '#fbbf24' },
  { value: 'fanfare', label: 'แตรวง', color: '#34d399' },
]

const goalPct = computed(() => {
  const p = profile.value
  if (!p || p.goal_amount <= 0) return 0
  return Math.min(100, Math.round((p.goal_raised / p.goal_amount) * 100))
})
const hasGoal = computed(() => !!profile.value && profile.value.goal_amount > 0)

const displayName = computed(() => profile.value?.display_name ?? 'Donate Me')
// Phase 9: ธีมหน้าโดเนต (rose = เดิม) — override ตัวแปรสีบน root ของหน้า
const pageTheme = computed(() => (profile.value?.page_theme as string) || 'rose')

// เหตุผลที่สลิปถูกปฏิเสธ (status เปลี่ยนเป็น rejected แล้ว — composable หยุด poll ตอนนั้น)
const statusNote = ref('')
watch(status, async (s) => {
  if (s === 'rejected' && current.value) {
    try {
      const res = await api.getDonation(current.value.id)
      statusNote.value = res.review_note
    } catch {
      /* เหตุผลไม่จำเป็นต้องมี */
    }
  }
})
const initial = computed(() => {
  const n = profile.value?.display_name
  return n ? n[0] : '🎮'
})

onMounted(() => {
  applyTheme()
  // ไม่ระบุสตรีมเมอร์ → หน้าแรกคือทำเนียบสตรีมเมอร์ (Demo Mode ถอดออกแล้ว ตาม ADR-0002)
  if (!targetUsername.value) {
    router.replace('/')
    return
  }
  void refreshData()
})

function timeAgo(ms: number): string {
  const diff = Math.floor((Date.now() - ms) / 1000)
  if (diff < 60) return `${diff} วินาทีที่แล้ว`
  if (diff < 3600) return `${Math.floor(diff / 60)} นาทีที่แล้ว`
  if (diff < 86400) return `${Math.floor(diff / 3600)} ชั่วโมงที่แล้ว`
  return `${Math.floor(diff / 86400)} วันที่แล้ว`
}

async function submit() {
  error.value = ''
  if (!amount.value || amount.value < 1) return (error.value = 'กรุณากรอกจำนวนเงินอย่างน้อย 1 บาท')
  if (!anonymous.value && !name.value.trim()) return (error.value = 'กรุณากรอกชื่อ หรือเลือกไม่แสดงชื่อ')

  submitting.value = true
  try {
    const donorName = anonymous.value ? 'ไม่ระบุชื่อ' : name.value.trim()
    const res = await api.createDonation({
      name: donorName,
      amount: amount.value,
      message: message.value,
      sound: sound.value,
      username: targetUsername.value,
    })
    current.value = {
      id: res.id,
      qrUrl: res.qr_url,
      payUrl: res.pay_url,
      amount: amount.value,
      sound: sound.value,
      name: donorName,
      mode: res.mode,
    }
    start(res.id, { sound: sound.value, name: donorName, amount: amount.value, message: message.value })
    stage.value = 'pay'
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'เกิดข้อผิดพลาด'
  } finally {
    submitting.value = false
  }
}

function backToForm() {
  clearTimeout(resetTimer)
  stop()
  status.value = null
  current.value = null
  amount.value = null // เริ่มรอบใหม่ — คงชื่อไว้ แต่เคลียร์ยอด/ข้อความ
  message.value = ''
  stage.value = 'form'
}

onUnmounted(() => clearTimeout(resetTimer))
</script>

<template>
  <div class="page" :class="`ptheme-${pageTheme}`">
    <SiteTopbar full />

    <main class="shell">
      <!-- ===== Hero สตรีมเมอร์ (มีรูปปกถ้าตั้งค่า — Phase 9) ===== -->
      <section v-if="!loadError" class="card hero" :class="{ 'has-cover': !!profile?.cover_url }"
        :style="profile?.cover_url ? { backgroundImage: `url(${profile.cover_url})` } : undefined">
        <div v-if="profile?.cover_url" class="cover-shade" aria-hidden="true" />
        <div class="avatar"><span>{{ initial }}</span></div>
        <div class="hero-info">
          <h1>{{ displayName }}</h1>
          <p class="bio">
            <template v-if="profile">สนับสนุน @{{ profile.username }} — ทุกกำลังใจช่วยให้สตรีมต่อเนื่องและสนุกยิ่งขึ้น 💜</template>
          </p>
          <!-- Phase 9: ลิงก์โซเชียล -->
          <div v-if="profile?.socials?.length" class="socials">
            <a
              v-for="s in profile.socials"
              :key="s.kind"
              :href="s.url"
              target="_blank"
              rel="noopener"
              class="social-chip"
              :class="`soc-${s.kind}`"
              :title="s.kind"
            >
              <span class="soc-ico">{{ { facebook: 'f', youtube: '▶', twitch: '🎮', tiktok: '♪', x: '𝕏' }[s.kind] ?? '🔗' }}</span>
              <span class="soc-label">{{ { facebook: 'Facebook', youtube: 'YouTube', twitch: 'Twitch', tiktok: 'TikTok', x: 'X' }[s.kind] ?? s.kind }}</span>
            </a>
          </div>
          <RouterLink class="back-link" to="/">← ทำเนียบสตรีมเมอร์ทั้งหมด</RouterLink>
          <div v-if="hasGoal" class="goal-box">
            <div class="goal-head">
              <span class="goal-title">🎯 เป้าหมายการสนับสนุน</span>
              <span class="goal-pct">{{ goalPct }}%</span>
            </div>
            <div
              class="goal-bar"
              role="progressbar"
              :aria-valuenow="goalPct"
              aria-valuemin="0"
              aria-valuemax="100"
            >
              <div class="goal-fill" :style="{ width: goalPct + '%' }" />
            </div>
            <div class="goal-foot">
              <span>฿{{ (profile?.goal_raised ?? 0).toLocaleString() }} สะสมแล้ว</span>
              <span>เป้าหมาย ฿{{ profile?.goal_amount.toLocaleString() }}</span>
            </div>
          </div>
        </div>
      </section>

      <!-- ===== เกี่ยวกับฉัน (Phase 9) ===== -->
      <section v-if="profile?.about_text" class="card about-card">
        <h2>💬 เกี่ยวกับ{{ displayName }}</h2>
        <p class="about-text">{{ profile.about_text }}</p>
      </section>

      <p v-if="loadError" class="load-error">{{ loadError }}</p>

      <!-- ===== การ์ดโดเนต / QR / ผลลัพธ์ (in-place swap) ===== -->
      <section class="card main-card">
        <!-- ฟอร์ม -->
        <template v-if="stage === 'form'">
          <div class="card-head">
            <h2>ส่งกำลังใจให้{{ profile ? displayName : 'สตรีมเมอร์' }}</h2>
            <span class="badge" :class="profile?.accepts_slip ? 'badge-green' : 'badge-pink'">
              {{ profile?.accepts_slip ? 'โอนตรง 0% ค่าธรรมเนียม' : 'PromptPay' }}
            </span>
          </div>

          <label class="lbl">จำนวนเงิน (บาท)</label>
          <div class="presets">
            <button
              v-for="p in presets"
              :key="p.v"
              type="button"
              class="preset"
              :class="{ active: amount === p.v }"
              @click="amount = p.v"
            >
              <span class="preset-label">{{ p.emoji }} {{ p.label }}</span>
              <b>฿{{ p.v }}</b>
            </button>
          </div>
          <div class="amount-input">
            <span class="thb">฿</span>
            <input
              v-model.number="amount"
              type="number"
              min="1"
              max="100000"
              placeholder="ระบุจำนวนเอง"
              aria-label="จำนวนเงินบาท"
            />
          </div>

          <div class="name-row">
            <div class="name-field">
              <label class="lbl" for="donor-name">ชื่อของคุณ</label>
              <input id="donor-name" v-model="name" class="input" placeholder="NongSomZa_77" maxlength="50" :disabled="anonymous" />
            </div>
            <label class="anon">
              <input v-model="anonymous" type="checkbox" />
              <span>ไม่แสดงชื่อ</span>
            </label>
          </div>

          <details class="extra" open>
            <summary>ข้อความ &amp; เสียงประกอบ <em>(ไม่บังคับ)</em></summary>
            <div class="msg-head">
              <label class="lbl" for="donor-msg">ข้อความถึงสตรีมเมอร์</label>
              <span class="counter">{{ message.length }}/150</span>
            </div>
            <textarea
              id="donor-msg"
              v-model="message"
              class="input"
              rows="3"
              placeholder="สู้ ๆ นะพี่! 💗"
              maxlength="150"
            ></textarea>
            <p class="tts-note">🔊 ข้อความนี้จะถูกอ่านออกเสียงบนสตรีมทุกครั้ง — ถ้าไม่กรอก ระบบใช้ข้อความเริ่มต้นให้</p>

            <label class="lbl">เสียงเมื่อ Alert เด้ง</label>
            <div class="sound-chips">
              <button
                v-for="s in sounds"
                :key="s.value"
                type="button"
                class="sound-chip"
                :class="{ active: sound === s.value }"
                :aria-pressed="sound === s.value"
                @click="sound = s.value"
              >
                <i :style="{ background: s.color }" /> {{ s.label }}
              </button>
            </div>

            <div class="preview-strip">
              <div class="preview-label">ตัวอย่าง Alert</div>
              <div class="preview-body">
                <b>{{ anonymous ? 'ไม่ระบุชื่อ' : name || 'ชื่อของคุณ' }}</b>
                สนับสนุน <em>฿{{ (amount || 0).toLocaleString() }}</em>
                <template v-if="message"> : "{{ message }}"</template>
              </div>
            </div>
          </details>

          <p v-if="error" class="error" role="alert">{{ error }}</p>

          <button class="btn-primary cta" :disabled="submitting" @click="submit">
            {{ submitting ? 'กำลังสร้าง QR...' : `โดเนต ${amount ? '฿' + amount.toLocaleString() : ''}` }}
          </button>
        </template>

        <!-- QR รอชำระเงิน -->
        <template v-else-if="stage === 'pay' && current">
          <div class="card-head">
            <h2>สแกนเพื่อส่งกำลังใจ</h2>
            <span class="badge badge-pink">PromptPay</span>
          </div>
          <p class="pay-sub">เปิดแอปธนาคาร → สแกน → ยืนยันยอด ฿{{ current.amount.toLocaleString() }}</p>

          <div class="qr-box">
            <div class="qr-head">PromptPay พร้อมเพย์</div>
            <div class="qr-body">
              <img v-if="current.qrUrl" :src="current.qrUrl" alt="PromptPay QR" class="qr" />
              <span class="qr-heart">❤️</span>
            </div>
            <div class="qr-amount">฿{{ current.amount.toLocaleString() }}.00</div>
          </div>

          <!-- โอนตรง: เงินเข้าบัญชีสตรีมเมอร์เลย — แนบสลิปเพื่อยืนยัน -->
          <template v-if="current.mode === 'direct'">
            <SlipUpload :donation-id="current.id" />
            <p v-if="status === 'awaiting_review'" class="review-note" role="status">
              🕒 ได้รับสลิปแล้ว — รอเจ้าของสตรีมตรวจสอบ (หน้านี้จะเฉลิมฉลองให้ทันทีที่อนุมัติ)
            </p>
          </template>

          <div class="expire-box">
            <b>⏳ QR หมดอายุใน {{ countdown }} นาที</b>
            <p>
              {{ current.mode === 'direct'
                ? 'เงินโอนเข้าบัญชีของสตรีมเมอร์โดยตรง ไม่ผ่านเว็บเรา — โอนแล้วแนบสลิปได้เลย'
                : 'จ่ายสำเร็จเมื่อไหร่ หน้านี้จะเฉลิมฉลองให้ทันที และ Alert จะเด้งในฉากสตรีมภายในไม่กี่วินาที' }}
            </p>
          </div>

          <div class="actions">
            <a v-if="current.payUrl" :href="current.payUrl" target="_blank" class="btn-primary">
              🧪 จำลองการชำระเงิน
            </a>
            <button class="btn-ghost" @click="backToForm">← ยกเลิก</button>
          </div>
          <p class="hint">{{ current.mode === 'mock' ? 'ระบบจำลอง — ยังไม่มีการตัดเงินจริง' : 'โอนตรงเข้าบัญชีสตรีมเมอร์ ไม่มีค่าธรรมเนียมเพิ่ม' }}</p>
        </template>

        <!-- สำเร็จ -->
        <template v-else-if="stage === 'paid'">
          <div class="result">
            <div class="big">🎉</div>
            <h2>โดเนตสำเร็จ!</h2>
            <p class="pay-sub">ขอบคุณมาก ๆ ที่สนับสนุน ❤️ Alert เด้งบนฉากสตรีมแล้ว</p>
            <div class="badge badge-green">จ่ายเงินสำเร็จ</div>
            <p class="muted small reset-note">ระบบจะกลับไปหน้าโดเนตให้อัตโนมัติ</p>
          </div>
        </template>

        <!-- ไม่สำเร็จ / หมดอายุ / สลิปถูกปฏิเสธ -->
        <template v-else>
          <div class="result">
            <div class="big">{{ status === 'rejected' ? '🚫' : '⌛' }}</div>
            <h2>{{ status === 'rejected' ? 'สลิปถูกปฏิเสธ' : 'QR หมดอายุหรือรายการไม่สำเร็จ' }}</h2>
            <p v-if="status === 'rejected' && statusNote" class="pay-sub">เหตุผล: {{ statusNote }}</p>
            <p class="pay-sub">ไม่มีการตัดเงินผ่านเว็บเรา — ลองสร้าง QR ใหม่ได้เลย</p>
            <button class="btn-primary" @click="backToForm">← กลับไปโดเนตใหม่</button>
          </div>
        </template>
      </section>

      <!-- ===== Social proof: Top 5 (บน) ===== -->
      <section v-if="topDonators.length" class="card social-card">
        <div class="card-head">
          <h2>🏆 ผู้สนับสนุนสูงสุดเดือนนี้</h2>
          <span class="muted">Top 5</span>
        </div>
        <ol class="board">
          <li v-for="(t, i) in topDonators" :key="t.donor_name">
            <span class="rank" :class="`rank-${i + 1}`">{{ i + 1 }}</span>
            <div class="board-info">
              <b>{{ t.donor_name }}</b>
              <span>{{ t.count }} ครั้ง</span>
            </div>
            <span class="board-amount">฿{{ t.total.toLocaleString() }}</span>
          </li>
        </ol>
      </section>

      <!-- ===== Social proof: กำลังใจล่าสุด + โหลดเพิ่ม (ล่าง) ===== -->
      <section v-if="recent.length || profile" class="card social-card">
        <div class="card-head">
          <h2>⚡ กำลังใจล่าสุด</h2>
          <span class="badge badge-green">● สด</span>
        </div>
        <div v-if="recent.length" class="feed">
          <div v-for="(r, i) in recent" :key="`${r.donor_name}-${r.paid_at}-${i}`" class="feed-item">
            <span class="feed-icon">{{ ['❤️', '⭐', '⚡', '💜', '🎉'][i % 5] }}</span>
            <div>
              <b>{{ r.donor_name }}</b>
              <em class="feed-amt">฿{{ r.amount.toLocaleString() }}</em>
              <span class="muted small">· {{ timeAgo(r.paid_at) }}</span>
              <p v-if="r.message">"{{ r.message }}"</p>
            </div>
          </div>
        </div>
        <p v-else class="muted empty">ยังไม่มีรายการ — เป็นคนแรกที่ส่งกำลังใจสิ!</p>

        <button
          v-if="recentHasMore"
          class="btn-ghost more-btn"
          :disabled="recentLoading"
          @click="loadRecent()"
        >
          {{ recentLoading ? 'กำลังโหลด...' : '⬇ โหลดกำลังใจเพิ่มอีก' }}
        </button>
        <p v-else-if="recent.length >= PAGE_SIZE" class="muted small more-end">แสดงครบทุกรายการแล้ว · รวม {{ recent.length }} รายการ</p>
      </section>

      <footer class="footer">
        <span><b>Donate Me</b> ❤️ แพลตฟอร์มสนับสนุนสตรีมเมอร์คนไทย</span>
        <span class="muted">© 2026 Donate Me</span>
      </footer>
    </main>

    <!-- CTA ติดขอบล่างบนมือถือ -->
    <div v-if="stage === 'form'" class="sticky-cta">
      <button class="btn-primary" :disabled="submitting" @click="submit">
        {{ submitting ? 'กำลังสร้าง QR...' : `โดเนต ${amount ? '฿' + amount.toLocaleString() : ''}` }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.shell {
  max-width: 680px;
  margin: 0 auto;
  padding: 26px 20px 48px;
  display: grid;
  gap: 18px;
}

/* hero */
.hero {
  display: flex;
  gap: 18px;
  align-items: flex-start;
  padding: 24px;
}
/* ===== Phase 9: รูปปก ===== */
.hero.has-cover {
  position: relative;
  background-size: cover;
  background-position: center;
  overflow: hidden;
  padding-top: 130px; /* เผื่อที่ให้ภาพปกด้านบน */
}
.cover-shade {
  position: absolute;
  inset: 0;
  background: linear-gradient(180deg, rgba(8, 12, 24, 0.62) 0%, rgba(8, 12, 24, 0.78) 55%, var(--bg-card) 90%);
  pointer-events: none;
}
.hero.has-cover > * { position: relative; z-index: 1; }

/* ===== Phase 9: โซเชียล ===== */
.socials { display: flex; gap: 8px; flex-wrap: wrap; margin: 10px 0 4px; }
.social-chip {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 5px 12px 5px 6px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-dim);
  font-size: 12.5px;
  text-decoration: none;
  transition: border-color 0.15s, transform 0.15s;
}
.social-chip:hover { border-color: var(--border-bright); transform: translateY(-1px); }
.soc-ico {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11.5px;
  font-weight: 800;
  color: #fff;
  font-family: var(--font-head);
}
.soc-facebook .soc-ico { background: #1877f2; }
.soc-youtube .soc-ico { background: #ff0000; }
.soc-twitch .soc-ico { background: #9146ff; }
.soc-tiktok .soc-ico { background: #010101; border: 1px solid #444; }
.soc-x .soc-ico { background: #1d1d1d; border: 1px solid #444; }

/* ===== Phase 9: About ===== */
.about-card { padding: 22px; }
.about-card h2 { font-size: 16px; margin-bottom: 8px; }
.about-text { color: var(--text-dim); font-size: 14px; line-height: 1.75; white-space: pre-line; margin: 0; }

/* ===== Phase 9: ธีมหน้า (override ตัวแปรสี — rose = default เดิม) ===== */
.ptheme-mint { --primary: #10b981; --primary-soft: rgba(16, 185, 129, 0.12); }
.ptheme-midnight { --primary: #818cf8; --primary-soft: rgba(129, 140, 248, 0.12); --bg: #0b1020; }
.ptheme-retro { --primary: #f59e0b; --primary-soft: rgba(245, 158, 11, 0.12); }
.ptheme-retro .hero, .ptheme-retro .main-card { border-style: dashed; }

.avatar {
  flex-shrink: 0;
  width: 76px;
  height: 76px;
  border-radius: 22px;
  overflow: hidden;
  background: linear-gradient(135deg, var(--primary), #a855f7);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 34px;
  font-weight: 700;
  color: #fff;
}
.hero-info { flex: 1; min-width: 0; }
h1 { font-size: 25px; font-weight: 700; }
.bio { margin-top: 6px; color: var(--text-dim); font-size: 14px; line-height: 1.65; }
.back-link {
  display: inline-block;
  margin-top: 10px;
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-faint);
  text-decoration: none;
}
.back-link:hover { color: var(--primary); }
.goal-box {
  margin-top: 14px;
  padding: 13px 15px;
  border-radius: var(--radius-lg);
  background: var(--bg-card-2);
  border: 1px solid var(--border);
}
.goal-head { display: flex; justify-content: space-between; font-size: 13.5px; font-weight: 700; }
.goal-pct { color: var(--emerald); }
.goal-bar {
  height: 8px;
  border-radius: 999px;
  background: var(--border);
  margin: 10px 0 8px;
  overflow: hidden;
}
.goal-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, var(--primary), var(--emerald));
  transition: width 0.6s ease;
}
.goal-foot { display: flex; justify-content: space-between; font-size: 12px; color: var(--text-faint); }
.load-error {
  text-align: center;
  padding: 28px;
  color: var(--text-dim);
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
}

/* การ์ดหลัก */
.main-card { padding: 24px; }
.card-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; }
.card-head h2 { font-size: 18px; font-weight: 700; }
.muted { color: var(--text-faint); font-size: 12.5px; }
.small { font-size: 11.5px; }

.lbl {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-dim);
  margin: 14px 0 7px;
}
.presets { display: grid; grid-template-columns: repeat(4, 1fr); gap: 9px; }
.preset {
  padding: 11px 8px;
  border-radius: var(--radius-md);
  text-align: left;
  border: 1px solid var(--border);
  background: var(--bg-input);
  color: var(--text);
  cursor: pointer;
  transition: all 0.15s;
  font-family: var(--font);
}
.preset-label { display: block; font-size: 10.5px; color: var(--text-faint); margin-bottom: 3px; white-space: nowrap; }
.preset b { font-size: 16px; font-weight: 700; }
.preset:hover { border-color: var(--border-bright); }
.preset.active {
  background: linear-gradient(135deg, #fb7185, var(--primary));
  border-color: transparent;
  box-shadow: 0 6px 20px rgba(244, 63, 94, 0.4);
}
.preset.active .preset-label { color: rgba(255, 255, 255, 0.85); }
.amount-input {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  padding: 0 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}
.amount-input:focus-within { border-color: var(--primary); box-shadow: 0 0 0 3px var(--primary-soft); }
.amount-input .thb { color: var(--text-faint); }
.amount-input input {
  flex: 1;
  border: none;
  background: transparent;
  padding: 12px 0;
  color: var(--text);
  font-size: 15px;
  font-family: var(--font);
  outline: none;
}
.name-row { display: flex; align-items: flex-end; gap: 14px; }
.name-field { flex: 1; }
.anon {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 13px;
  cursor: pointer;
  font-size: 12.5px;
  color: var(--text-dim);
  white-space: nowrap;
}
.anon input { width: auto; accent-color: var(--primary); }

/* ส่วนยุบ: ข้อความ + เสียง */
.extra {
  margin-top: 16px;
  padding: 12px 14px;
  border-radius: var(--radius-lg);
  background: var(--bg-card-2);
  border: 1px solid var(--border);
}
.extra summary {
  cursor: pointer;
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-dim);
  list-style: none;
}
.extra summary::-webkit-details-marker { display: none; }
.extra summary::before { content: '▸ '; color: var(--primary); }
.extra[open] summary::before { content: '▾ '; }
.extra summary em { font-style: normal; font-size: 11.5px; color: var(--text-faint); }
.extra[open] summary { margin-bottom: 4px; }
.msg-head { display: flex; justify-content: space-between; align-items: center; }
.counter { font-size: 11px; color: var(--text-faint); }
.tts-note { margin-top: 7px; font-size: 11.5px; color: var(--blue); line-height: 1.5; }
textarea { resize: vertical; }
.sound-chips { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
.sound-chip {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 9px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-input);
  color: var(--text-dim);
  font-size: 12.5px;
  font-family: var(--font);
  cursor: pointer;
  transition: all 0.15s;
}
.sound-chip i { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.sound-chip:hover { border-color: var(--border-bright); color: var(--text); }
.sound-chip.active { border-color: var(--primary); color: var(--text); background: var(--primary-soft); }
.preview-strip {
  margin-top: 14px;
  padding: 12px 14px;
  border-radius: var(--radius-md);
  background: var(--bg-input);
  border: 1px dashed var(--border-bright);
}
.preview-label { font-size: 11px; color: var(--text-faint); margin-bottom: 6px; }
.preview-body { font-size: 13px; color: var(--text-dim); line-height: 1.6; }
.preview-body b { color: var(--primary); }
.preview-body em { font-style: normal; color: var(--gold); font-weight: 700; }
.error { color: var(--primary); font-size: 13px; margin-top: 12px; text-align: center; }

.cta { width: 100%; margin-top: 18px; padding: 15px; font-size: 16.5px; }

/* QR */
.pay-sub { color: var(--text-dim); font-size: 14px; line-height: 1.6; }
.review-note {
  margin-top: 14px;
  padding: 12px 14px;
  border-radius: var(--radius-md);
  background: rgba(16, 185, 129, 0.08);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: var(--emerald);
  font-size: 13px;
  text-align: center;
  line-height: 1.6;
}
.qr-box {
  max-width: 300px;
  margin: 18px auto 0;
  border-radius: var(--radius-lg);
  overflow: hidden;
  background: #fff;
  color: #0f172a;
  box-shadow: 0 8px 30px rgba(15, 23, 42, 0.25);
  border: 1px solid #e2e8f0;
}
.qr-head { background: #1e3a8a; color: #fff; font-size: 13.5px; font-weight: 700; padding: 10px; text-align: center; }
.qr-body { position: relative; padding: 18px; text-align: center; }
.qr { width: 210px; height: 210px; image-rendering: pixelated; }
.qr-heart {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--primary);
  color: #fff;
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 3px solid #fff;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}
.qr-amount { font-size: 22px; font-weight: 700; padding: 6px 0 18px; text-align: center; }
.expire-box {
  margin: 16px auto 0;
  max-width: 380px;
  padding: 13px 15px;
  border-radius: var(--radius-md);
  background: var(--gold-soft);
  border: 1px solid rgba(251, 191, 36, 0.35);
  text-align: center;
}
.expire-box b { font-size: 13px; color: var(--gold); }
.expire-box p { font-size: 12px; color: var(--text-dim); margin-top: 5px; line-height: 1.6; }
.actions { display: flex; gap: 10px; justify-content: center; margin-top: 18px; flex-wrap: wrap; }
.actions .btn-primary { text-decoration: none; }
.hint { margin-top: 12px; font-size: 11.5px; color: var(--text-faint); text-align: center; }

/* ผลลัพธ์ */
.result { text-align: center; padding: 32px 16px; }
.big { font-size: 60px; margin-bottom: 8px; }
.result h2 { font-size: 24px; margin-bottom: 6px; }
.reset-note { margin-top: 14px; }

/* social proof: สอง section แนวตั้งเต็มความกว้าง (Top 5 บน / กำลังใจล่าสุด ลงล่าง) */
.social-card { padding: 22px; }
.board {
  list-style: none;
  display: grid;
  gap: 8px;
  grid-template-columns: 1fr;
}
.board li {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
  background: var(--bg-input);
}
.more-btn { width: 100%; margin-top: 14px; justify-content: center; }
.more-end { text-align: center; margin-top: 14px; }
.rank {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12.5px;
  font-weight: 700;
  background: var(--bg-card-2);
  color: var(--text-dim);
}
.rank-1 { background: var(--gold); color: #422006; }
.rank-2 { background: #cbd5e1; color: #334155; }
.rank-3 { background: #f59e0b; color: #431407; }
.board-info { flex: 1; display: flex; flex-direction: column; min-width: 0; }
.board-info b { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.board-info span { font-size: 11px; color: var(--text-faint); }
.board-amount { font-weight: 700; color: var(--gold); }
.empty { padding: 20px; text-align: center; }
.feed { display: grid; gap: 10px; }
.feed-item {
  display: flex;
  gap: 11px;
  padding: 11px 13px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
  background: var(--bg-input);
}
.feed-icon {
  width: 32px;
  height: 32px;
  border-radius: 10px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 15px;
  background: var(--bg-card-2);
}
.feed-item b { font-size: 13px; }
.feed-amt { font-style: normal; color: var(--emerald); font-weight: 700; font-size: 12.5px; margin-left: 6px; }
.feed-item p { font-size: 12px; color: var(--text-dim); margin-top: 3px; line-height: 1.55; }

.footer {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  padding-top: 16px;
  border-top: 1px solid var(--border);
  font-size: 12.5px;
  color: var(--text-dim);
}

/* CTA sticky เฉพาะจอแคบ */
.sticky-cta { display: none; }

@media (max-width: 640px) {
  .shell { padding: 18px 14px 96px; }
  .hero { flex-direction: column; padding: 20px; }
  .main-card { padding: 20px 16px; }
  h1 { font-size: 21px; }
  .presets { grid-template-columns: 1fr 1fr; }
  .sound-chips { grid-template-columns: 1fr 1fr; }
  .name-row { flex-direction: column; align-items: stretch; gap: 8px; }
  .anon { margin-bottom: 0; justify-content: flex-end; }
  .social-card { padding: 18px 14px; }
  .cta { display: none; }
  .sticky-cta {
    display: block;
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 20;
    padding: 12px 14px calc(12px + env(safe-area-inset-bottom));
    background: color-mix(in srgb, var(--bg) 82%, transparent);
    -webkit-backdrop-filter: blur(14px);
    backdrop-filter: blur(14px);
    border-top: 1px solid var(--border);
  }
  .sticky-cta .btn-primary { width: 100%; padding: 15px; font-size: 16px; }
}
</style>
