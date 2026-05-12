import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { Branch, Workspace } from '../types';

export const useRepoStore = defineStore('repo', () => {
  const current = ref<Workspace | null>(null);
  const branches = ref<Branch[]>([]);
  const base = ref<string | null>(null);
  const target = ref<string | null>(null);

  return { current, branches, base, target };
});
