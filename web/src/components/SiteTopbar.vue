<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { currentTheme, toggleTheme } from '../theme'

// Topbar กลางของทุกหน้า — โลโก้กดแล้วกลับหน้าหลักเสมอ
// full = หน้าโดเนต (มีเมนู + เข้าร่วม + ME), compact = หน้าย่อย (โลโก้ + สลับธีม)
defineProps<{ full?: boolean }>()
const router = useRouter()
const theme = ref(currentTheme())
function onToggle() {
  theme.value = toggleTheme()
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
      <a href="/#/" class="active">หน้าแรก</a>
      <a href="#">คู่มือสตรีมเมอร์</a>
      <a href="#" class="pill">จุดรับโดเนต</a>
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
.topnav a.pill {
  border: 1px solid var(--border-bright);
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
    flex-wrap: wrap;
    gap: 10px;
    padding: 10px 14px;
  }
  .logo {
    flex: 1;
  }
  .topnav {
    order: 3;
    width: 100%;
    overflow-x: auto;
  }
  .topnav a {
    white-space: nowrap;
    padding: 7px 12px;
    font-size: 12.5px;
  }
  .join-btn {
    padding: 7px 14px;
    font-size: 12.5px;
  }
}
</style>
