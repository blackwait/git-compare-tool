import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { Workspace } from '../types';
import { configApi } from '../api/config';

export const useWorkspacesStore = defineStore('workspaces', () => {
  const list = ref<Workspace[]>([]);
  const loading = ref(false);

  async function reload() {
    loading.value = true;
    try {
      list.value = await configApi.listWorkspaces();
    } finally {
      loading.value = false;
    }
  }

  return { list, loading, reload };
});
