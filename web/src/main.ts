import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import router from './router'
import { burstConfetti } from './confetti'

// เรียกใช้จาก console ได้ทุกหน้า: window.confetti() หรือ window.confetti(100)
window.confetti = burstConfetti

createApp(App).use(router).mount('#app')
