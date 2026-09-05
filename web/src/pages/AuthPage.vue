<script setup lang="ts">
import { ref } from 'vue'
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
      <div class="avatar">🎮</div>
      <h1>{{ mode === 'login' ? 'เข้าสู่ระบบ' : 'สมัครสมาชิก' }}</h1>
      <p class="subtitle">ระบบโดเนตสำหรับสตรีมเมอร์</p>

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
  background: linear-gradient(135deg, #1a1040 0%, #2d1b69 50%, #0f3460 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}
.card {
  width: 100%;
  max-width: 420px;
  background: rgba(255, 255, 255, 0.07);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 20px;
  padding: 36px 32px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  color: #fff;
}
.avatar {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  background: linear-gradient(135deg, #ff6ec7, #7873f5);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 34px;
  margin: 0 auto 16px;
}
h1 {
  text-align: center;
  font-size: 24px;
  margin-bottom: 4px;
}
.subtitle {
  text-align: center;
  opacity: 0.7;
  margin-bottom: 24px;
  font-size: 14px;
}
label {
  display: block;
  font-size: 13px;
  opacity: 0.8;
  margin: 14px 0 6px;
}
input {
  width: 100%;
  padding: 12px 14px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  background: rgba(0, 0, 0, 0.25);
  color: #fff;
  font-size: 15px;
  outline: none;
}
input:focus {
  border-color: #ff6ec7;
}
.error {
  color: #fda4af;
  font-size: 13px;
  margin-top: 12px;
  white-space: pre-wrap;
}
.submit-btn {
  width: 100%;
  margin-top: 22px;
  padding: 14px;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  background: linear-gradient(135deg, #ff6ec7, #7873f5);
  color: #fff;
  font-size: 16px;
  font-weight: 700;
}
.switch {
  text-align: center;
  margin-top: 18px;
  font-size: 14px;
  opacity: 0.8;
}
.switch a {
  color: #a5b4fc;
}
</style>
