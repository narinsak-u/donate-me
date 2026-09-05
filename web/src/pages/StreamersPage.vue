<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { api, type StreamerSummary } from '../api/auth'
import { applyTheme } from '../theme'
import SiteTopbar from '../components/SiteTopbar.vue'

const router = useRouter()
const streamers = ref<StreamerSummary[]>([])
const loading = ref(true)
const loadError = ref('')
const search = ref('')

onMounted(() => {
  applyTheme()
  api.streamers()
    .then((list) => (streamers.value = list))
    .catch(() => (loadError.value = 'โหลดรายชื่อสตรีมเมอร์ไม่สำเร็จ — ลองรีเฟรชอีกครั้ง'))
    .finally(() => (loading.value = false))
})

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return streamers.value
  return streamers.value.filter(
    (s) => s.display_name.toLowerCase().includes(q) || s.username.toLowerCase().includes(q),
  )
})

function pctOf(s: StreamerSummary): number {
  if (s.goal_amount <= 0) return 0
  return Math.min(100, Math.round((s.goal_raised / s.goal_amount) * 100))
}
</script>

<template>
  <div class="page">
    <SiteTopbar full />

    <main class="shell">
      <!-- ===== Hero ===== -->
      <section class="card hero">
        <h1>สนับสนุนสตรีมเมอร์คนโปรดของคุณ 💜</h1>
        <p class="sub">เลือกสตรีมเมอร์เพื่อส่งกำลังใจ — โดเนตผ่าน PromptPay แล้ว Alert เด้งสด ๆ บนฉากทันที</p>
        <input
          v-model="search"
          class="input search"
          type="search"
          placeholder="ค้นหาชื่อสตรีมเมอร์..."
          aria-label="ค้นหาสตรีมเมอร์"
        />
      </section>

      <p v-if="loadError" class="load-error" role="alert">{{ loadError }}</p>
      <p v-else-if="loading" class="muted loading">กำลังโหลดรายชื่อ...</p>

      <!-- ===== รายชื่อสตรีมเมอร์ ===== -->
      <section id="streamers" v-else-if="filtered.length" class="grid">
        <button
          v-for="s in filtered"
          :key="s.username"
          type="button"
          class="card streamer"
          @click="router.push(`/u/${s.username}`)"
        >
          <div class="head">
            <div class="avatar"><span>{{ s.display_name[0] }}</span></div>
            <div class="info">
              <b>{{ s.display_name }}</b>
              <span class="handle">@{{ s.username }}</span>
            </div>
          </div>
          <div class="stats">
            <em>฿{{ s.goal_raised.toLocaleString() }}</em>
            <span class="muted">{{ s.donor_count }} คนสนับสนุน</span>
          </div>
          <div v-if="s.goal_amount > 0" class="goal">
            <div class="goal-bar">
              <div class="goal-fill" :style="{ width: pctOf(s) + '%' }" />
            </div>
            <span class="muted small">{{ pctOf(s) }}% ของเป้า ฿{{ s.goal_amount.toLocaleString() }}</span>
          </div>
        </button>
      </section>

      <p v-else class="muted empty">ยังไม่มีสตรีมเมอร์ในระบบ — สมัครเป็นคนแรกได้ที่ "เข้าร่วม"</p>

      <footer class="footer">
        <span><b>Donate Me</b> ❤️ แพลตฟอร์มสนับสนุนสตรีมเมอร์คนไทย</span>
        <span class="muted">© 2026 Donate Me</span>
      </footer>
    </main>
  </div>
</template>

<style scoped>
.shell {
  max-width: 900px;
  margin: 0 auto;
  padding: 26px 20px 48px;
  display: grid;
  gap: 18px;
}
.hero { padding: 28px 26px 24px; }
h1 { font-size: 26px; font-weight: 700; }
.sub { margin-top: 7px; color: var(--text-dim); font-size: 14px; line-height: 1.65; }
.search { margin-top: 16px; max-width: 420px; }
.muted { color: var(--text-faint); font-size: 12.5px; }
.small { font-size: 11px; }
.loading, .empty { text-align: center; padding: 28px; }
.load-error {
  text-align: center;
  padding: 28px;
  color: var(--text-dim);
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 14px;
}
.streamer {
  padding: 18px;
  text-align: left;
  cursor: pointer;
  font-family: var(--font);
  color: var(--text);
  transition: transform 0.15s, border-color 0.15s;
}
.streamer:hover {
  transform: translateY(-2px);
  border-color: var(--border-bright);
}
.head { display: flex; align-items: center; gap: 12px; }
.avatar {
  width: 52px;
  height: 52px;
  border-radius: 16px;
  flex-shrink: 0;
  background: linear-gradient(135deg, var(--primary), #a855f7);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 23px;
  font-weight: 700;
  color: #fff;
}
.info { display: flex; flex-direction: column; min-width: 0; }
.info b { font-size: 15.5px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.handle { font-size: 12px; color: var(--text-faint); }
.stats { display: flex; align-items: baseline; gap: 8px; margin-top: 14px; }
.stats em { font-style: normal; font-size: 19px; font-weight: 700; color: var(--gold); }
.goal { margin-top: 10px; display: flex; flex-direction: column; gap: 5px; }
.goal-bar {
  height: 6px;
  border-radius: 999px;
  background: var(--border);
  overflow: hidden;
}
.goal-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, var(--primary), var(--emerald));
}

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

@media (max-width: 640px) {
  .shell { padding: 18px 14px 40px; }
  .hero { padding: 22px 18px; }
  h1 { font-size: 21px; }
  .grid { grid-template-columns: 1fr 1fr; }
}
@media (max-width: 460px) {
  .grid { grid-template-columns: 1fr; }
}
</style>
