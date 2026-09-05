import { createRouter, createWebHashHistory } from 'vue-router'

// hash history: serve จาก Axum ServeDir ได้เลย ไม่ต้องมี SPA fallback
// ลิงก์แชร์แบบ path /u/:username (server ใส่ OG tags ให้) → normalize เป็น hash ก่อน router บูต
const pathUser = location.pathname.match(/^\/u\/([a-z0-9_]+)\/?$/)
if (pathUser && !location.hash.startsWith('#/u/')) {
  history.replaceState(null, '', `/#/u/${pathUser[1]}${location.search}`)
}

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', component: () => import('../pages/StreamersPage.vue') },
    { path: '/u/:username', component: () => import('../pages/DonatePage.vue'), props: true },
    { path: '/pay/:id', component: () => import('../pages/PayPage.vue'), props: true },
    { path: '/auth', component: () => import('../pages/AuthPage.vue') },
    { path: '/dashboard', component: () => import('../pages/Dashboard.vue') },
  ],
})

// ลิงก์แชร์รูปแบบเก่า /?u=username#/ → พาไปหน้าโดเนตรูปแบบใหม่
router.beforeEach((to) => {
  const u = new URLSearchParams(location.search).get('u')
  if (u && to.path === '/') return { path: `/u/${u}` }
})

export default router
