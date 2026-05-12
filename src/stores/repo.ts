import { defineStore } from 'pinia';
import { ref, watch, nextTick } from 'vue';
import type { Branch, Workspace } from '../types';

const STORAGE_KEY = 'gct_branch_selection';

function loadAll(): Record<string, { base: string; target: string }> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function saveToStorage(wsId: string, base: string | null, target: string | null) {
  if (!wsId || (!base && !target)) return;
  const all = loadAll();
  all[wsId] = { base: base || '', target: target || '' };
  localStorage.setItem(STORAGE_KEY, JSON.stringify(all));
}

export function getSavedBranches(wsId: string): { base: string | null; target: string | null } {
  const all = loadAll();
  const entry = all[wsId];
  if (!entry) return { base: null, target: null };
  return { base: entry.base || null, target: entry.target || null };
}

export const useRepoStore = defineStore('repo', () => {
  const current = ref<Workspace | null>(null);
  const branches = ref<Branch[]>([]);
  const base = ref<string | null>(null);
  const target = ref<string | null>(null);
  const _pauseSave = ref(false);

  watch([base, target], ([b, t]) => {
    if (_pauseSave.value) return;
    if (!current.value) return;
    // 只有当 base 或 target 有值时才保存，避免清空覆盖
    if (b || t) {
      saveToStorage(current.value.id, b, t);
    }
  });

  function pauseSave() { _pauseSave.value = true; }
  function resumeSave() { nextTick(() => { _pauseSave.value = false; }); }

  return { current, branches, base, target, pauseSave, resumeSave };
});
