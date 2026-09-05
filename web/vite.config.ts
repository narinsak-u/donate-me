import vue from '@vitejs/plugin-vue'
import { resolve } from 'node:path'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  build: {
    rollupOptions: {
      // build 2 ชุด: SPA หลัก + overlay แบบ standalone สำหรับ OBS
      input: {
        main: resolve(__dirname, 'index.html'),
        overlay: resolve(__dirname, 'overlay.html'),
      },
    },
  },
  server: {
    proxy: {
      '/api': 'http://localhost:3000',
      '/mock': 'http://localhost:3000',
      '/events': 'http://localhost:3000',
    },
  },
})
