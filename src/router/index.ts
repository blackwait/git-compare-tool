import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  { path: '/', name: 'welcome', component: () => import('../views/Welcome.vue') },
  { path: '/repo/:id', name: 'repo', component: () => import('../views/RepoDetail.vue') },
  { path: '/settings', name: 'settings', component: () => import('../views/Settings.vue') },
];

export default createRouter({ history: createWebHashHistory(), routes });
