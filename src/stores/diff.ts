import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { BranchDiff, FileDiff } from '../types';
import { gitApi } from '../api/git';

export const useDiffStore = defineStore('diff', () => {
  const branchDiff = ref<BranchDiff | null>(null);
  const fileDiff = ref<FileDiff | null>(null);
  const viewMode = ref<'tree' | 'list'>('tree');
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadBranchDiff(repoId: string, base: string, target: string) {
    loading.value = true;
    error.value = null;
    branchDiff.value = null;
    try {
      branchDiff.value = await gitApi.diffBranches(repoId, base, target);
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function loadFileDiff(repoId: string, base: string, target: string, path: string) {
    fileDiff.value = null;
    try {
      fileDiff.value = await gitApi.fileDiff(repoId, base, target, path);
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  return { branchDiff, fileDiff, viewMode, loading, error, loadBranchDiff, loadFileDiff };
});
