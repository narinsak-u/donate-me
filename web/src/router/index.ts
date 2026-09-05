import { createRouter, createWebHashHistory } from 'vue-router'

// hash history: serve จาก Axum ServeDir ได้เลย ไม่ต้องมี SPA fallback
const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', component: () => import('../pages/DonatePage.vue') },
    { path: '/pay/:id', component: () => import('../pages/PayPage.vue'), props: true },
    { path: '/auth', component: () => import('../pages/AuthPage.vue') },
    { path: '/dashboard', component: () => import('../pages/Dashboard.vue') },
  ],
})

export default router
