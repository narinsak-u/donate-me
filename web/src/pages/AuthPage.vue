<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { authApi } from '../api/auth'

const router = useRouter()
const mode = ref<'login' | 'register'>('login')
const email = ref('')
const password = ref('')
const username = ref('')
const displayName = ref('')
const error = ref('')
const busy = ref(false)

onMounted(() => {
  document.documentElement.setAttribute('data-theme', localStorage.getItem('donateme_theme') ?? 'dark')
})

async function submit() {
  error.value = ''
  busy.value = true
  try {
    if (mode.value === 'login') {
      await authApi.login({ email: email.value, password: password.value })
    } else {
      await authApi.register({
        username: username.value,
        email: email.value,
        password: password.value,
        display_name: displayName.value || undefined,
      })
    }
    router.push('/dashboard')
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'เกิดข้อผิดพลาด'
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="page">
    <div class="card">
      <div class="avatar">💜</div>
      <h1>{{ mode === 'login' ? 'เข้าสู่ระบบ' : 'สมัครสมาชิก' }}</h1>
      <p class="subtitle">Creator Studio — ระบบโดเนตสำหรับสตรีมเมอร์</p>

      <template v-if="mode === 'register'">
        <label>username (ใช้เป็นลิงก์รับโดเนต)</label>
        <input v-model="username" placeholder="เช่น narin_plays" maxlength="20" />
        <label>ชื่อที่แสดง</label>
        <input v-model="displayName" placeholder="เช่น Narin Plays" maxlength="50" />
      </template>

      <label>อีเมล</label>
      <input v-model="email" type="email" placeholder="you@example.com" />

      <label>รหัสผ่าน</label>
      <input v-model="password" type="password" placeholder="อย่างน้อย 8 ตัวอักษร" @keyup.enter="submit" />

      <p v-if="error" class="error">{{ error }}</p>

      <button class="submit-btn" :disabled="busy" @click="submit">
        {{ busy ? 'กำลัง...' : mode === 'login' ? 'เข้าสู่ระบบ' : 'สมัครสมาชิก' }}
      </button>

      <p class="switch">
        {{ mode === 'login' ? 'ยังไม่มีบัญชี?' : 'มีบัญชีแล้ว?' }}
        <a href="#" @click.prevent="mode = mode === 'login' ? 'register' : 'login'">
          {{ mode === 'login' ? 'สมัครสมาชิก' : 'เข้าสู่ระบบ' }}
        </a>
      </p>
    </div>
  </div>
</template>

<style scoped>
.page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: var(--bg);
}
.card {
  width: 100%;
  max-width: 420px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  padding: 38px 34px;
  box-shadow: var(--shadow-card);
}
.avatar {
  width: 72px;
  height: 72px;
  border-radius: 22px;
  background: linear-gradient(135deg, #fb7185, var(--primary));
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  margin: -62px auto 16px;
  border: 4px solid var(--bg);
  box-shadow: 0 8px 24px rgba(244, 63, 94, 0.35);
}
h1 {
  text-align: center;
  font-size: 26px;
}
.subtitle {
  text-align: center;
  color: var(--text-dim);
  margin: 6px 0 24px;
  font-size: 14px;
}
label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-dim);
  margin: 14px 0 6px;
}
input {
  width: 100%;
  padding: 13px 15px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
  background: var(--bg-input);
  color: var(--text);
  font-size: 15px;
  font-family: var(--font-body);
  outline: none;
  transition: border 0.2s, box-shadow 0.2s;
}
input:focus {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-soft);
}
.error {
  color: var(--primary);
  font-size: 13px;
  margin-top: 12px;
  white-space: pre-wrap;
}
.submit-btn {
  width: 100%;
  margin-top: 24px;
  padding: 15px;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  background: linear-gradient(135deg, #fb7185, var(--primary));
  color: #fff;
  font-size: 17px;
  font-weight: 700;
  font-family: var(--font-head);
  box-shadow: 0 8px 26px rgba(244, 63, 94, 0.3);
  transition: transform 0.15s;
}
.submit-btn:hover:not(:disabled) {
  transform: translateY(-2px);
}
.switch {
  text-align: center;
  margin-top: 20px;
  font-size: 14px;
  color: var(--text-dim);
}
.switch a {
  color: var(--primary);
}
</style>
