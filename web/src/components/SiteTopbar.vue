<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { currentTheme, toggleTheme } from '../theme'

// Topbar กลางของทุกหน้า — โลโก้กดแล้วกลับหน้าหลักเสมอ
// full = หน้าสาธารณะ (มีเมนู + เข้าร่วม + ME), compact = หน้าย่อย (โลโก้ + สลับธีม)
defineProps<{ full?: boolean }>()
const router = useRouter()
const route = useRoute()
const theme = ref(currentTheme())
function onToggle() {
  theme.value = toggleTheme()
}

const isHome = computed(() => route.path === '/')

// เมนู "จุดรับโดเนต" — ไปหน้าแรกแล้วเลื่อนถึงรายชื่อสตรีมเมอร์
async function goDonateSpot() {
  if (route.path !== '/') await router.push('/')
  requestAnimationFrame(() =>
    document.getElementById('streamers')?.scrollIntoView({ behavior: 'smooth' }),
  )
}
</script>

<template>
  <header class="topbar">
    <div class="logo" role="link" title="กลับหน้าหลัก" @click="router.push('/')">
      <span class="logo-icon">❤️</span>
      <b>Donate Me</b>
      <span class="logo-heart">❤️</span>
    </div>

    <nav v-if="full" class="topnav">
      <a href="/#/" :class="{ active: isHome }">หน้าแรก</a>
      <a href="/#/dashboard?tab=settings" :class="{ active: route.path === '/dashboard' }">คู่มือสตรีมเมอร์</a>
      <button type="button" class="nav-btn" @click="goDonateSpot">จุดรับโดเนต</button>
    </nav>

    <div class="topbar-right">
      <slot />
      <button class="theme-toggle" title="สลับธีม" @click="onToggle">{{ theme === 'dark' ? '☀️' : '🌙' }}</button>
      <a v-if="full" href="/#/dashboard" class="join-btn">เข้าร่วม</a>
      <div v-if="full" class="me-avatar">ME</div>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 28px;
  padding: 12px 32px;
  background: color-mix(in srgb, var(--bg) 85%, transparent);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--border);
}
.logo {
  display: flex;
  align-items: center;
  gap: 7px;
  font-family: var(--font-head);
  font-size: 17px;
  cursor: pointer;
  user-select: none;
}
.logo:hover b {
  color: var(--primary);
}
.logo-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 11px;
  font-size: 16px;
  background: var(--primary-soft);
}
.logo-heart {
  font-size: 11px;
}
.topnav {
  display: flex;
  gap: 6px;
  flex: 1;
}
.topnav a {
  padding: 8px 15px;
  border-radius: 999px;
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-dim);
  text-decoration: none;
}
.topnav a.active {
  color: var(--primary);
  background: var(--primary-soft);
}
.topnav a.pill,
.nav-btn {
  border: 1px solid var(--border-bright);
}
.nav-btn {
  padding: 8px 15px;
  border-radius: 999px;
  background: transparent;
  color: var(--text);
  font-size: 13.5px;
  font-weight: 600;
  font-family: var(--font);
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;
}
.nav-btn:hover {
  border-color: var(--primary);
  color: var(--primary);
}
.topbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-left: auto;
}
.theme-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 50%;
  border: 1px solid var(--border);
  background: var(--bg-card);
  font-size: 17px;
  cursor: pointer;
  transition: transform 0.2s;
}
.theme-toggle:hover {
  transform: rotate(20deg) scale(1.08);
}
.join-btn {
  padding: 9px 20px;
  border-radius: 999px;
  border: 1px solid var(--border-bright);
  color: var(--text);
  font-size: 13.5px;
  font-weight: 700;
  text-decoration: none;
}
.me-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--primary), #fb7185);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 800;
  color: #fff;
}
@media (max-width: 640px) {
  .topbar {
    gap: 10px;
    padding: 10px 14px;
  }
  .logo {
    flex: 1;
    min-width: 0;
  }
  .logo b {
    white-space: nowrap;
  }
  /* มือถือ: ซ่อนเมนูการตลาด — ผู้บริจาคไม่ใช้ และเป็นหน้า minimal */
  .topnav {
    display: none;
  }
  .join-btn {
    padding: 7px 14px;
    font-size: 12.5px;
    white-space: nowrap;
  }
}
</style>
