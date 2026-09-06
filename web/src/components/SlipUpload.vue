<script setup lang="ts">
// Phase 7: แนบสลิปหลังโอนเงินเข้าบัญชีสตรีมเมอร์ตรง — ใช้ร่วมกันบนหน้าโดเนตและหน้า /pay/:id
import { ref } from 'vue'

const props = defineProps<{ donationId: string }>()
const emit = defineEmits<{ uploaded: [] }>()

const file = ref<File | null>(null)
const preview = ref('')
const uploading = ref(false)
const error = ref('')
const done = ref(false)
const inputEl = ref<HTMLInputElement | null>(null)

function pick(f: File | null) {
  error.value = ''
  done.value = false
  if (preview.value) URL.revokeObjectURL(preview.value)
  preview.value = ''
  file.value = null
  if (!f) return
  if (!['image/png', 'image/jpeg'].includes(f.type)) {
    error.value = 'รับเฉพาะไฟล์รูป PNG / JPG'
    return
  }
  if (f.size > 5 * 1024 * 1024) {
    error.value = 'ไฟล์ใหญ่เกิน 5MB'
    return
  }
  file.value = f
  preview.value = URL.createObjectURL(f)
}

async function upload() {
  if (!file.value || uploading.value) return
  uploading.value = true
  error.value = ''
  try {
    const fd = new FormData()
    fd.append('file', file.value)
    const res = await fetch(`/api/donate/${props.donationId}/slip`, { method: 'POST', body: fd })
    if (!res.ok) throw new Error((await res.text()) || 'แนบสลิปไม่สำเร็จ')
    done.value = true
    emit('uploaded')
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'แนบสลิปไม่สำเร็จ'
  } finally {
    uploading.value = false
  }
}
</script>

<template>
  <div class="slip">
    <p class="slip-title">🧾 <b>แนบสลิปหลังโอน</b> — เจ้าของสตรีมจะตรวจแล้วเปิด Alert ให้ทันที</p>

    <div v-if="!preview" class="drop" role="button" tabindex="0" @click="inputEl?.click()" @keydown.enter="inputEl?.click()">
      <span class="drop-icon">📷</span>
      <span>แตะเพื่อเลือกรูปสลิป (PNG/JPG ไม่เกิน 5MB)</span>
    </div>
    <input ref="inputEl" type="file" accept="image/png,image/jpeg" class="visually-hidden" @change="pick($event.target.files?.[0] ?? null)" />

    <div v-if="preview" class="preview-row">
      <img :src="preview" alt="สลิปที่เลือก" class="preview" />
      <div class="preview-actions">
        <button class="btn-primary" :disabled="uploading || done" @click="upload">
          {{ done ? '✅ ส่งสลิปแล้ว' : uploading ? 'กำลังส่ง...' : 'ส่งสลิป' }}
        </button>
        <button class="btn-ghost" :disabled="uploading" @click="inputEl?.click()">เปลี่ยนรูป</button>
      </div>
    </div>

    <p v-if="error" class="slip-error" role="alert">{{ error }}</p>
  </div>
</template>

<style scoped>
.slip { margin-top: 14px; }
.slip-title { font-size: 13px; color: var(--text-dim); margin-bottom: 10px; line-height: 1.6; }
.slip-title b { color: var(--text); }
.drop {
  border: 1.5px dashed var(--border-bright);
  border-radius: var(--radius-md);
  padding: 20px 14px;
  text-align: center;
  color: var(--text-dim);
  font-size: 13px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
  transition: border-color 0.15s, background 0.15s;
}
.drop:hover, .drop:focus-visible { border-color: var(--primary); background: var(--primary-soft); outline: none; }
.drop-icon { font-size: 24px; }
.visually-hidden { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
.preview-row { display: flex; gap: 12px; align-items: center; }
.preview {
  width: 92px;
  height: 120px;
  object-fit: cover;
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
}
.preview-actions { display: flex; flex-direction: column; gap: 8px; }
.slip-error { margin-top: 10px; color: var(--primary); font-size: 12.5px; }
</style>
