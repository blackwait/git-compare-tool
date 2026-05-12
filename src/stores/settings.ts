import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { configApi } from '../api/config';

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'dark' | 'light' | 'auto'>('dark');
  const defaultView = ref<'tree' | 'list'>('tree');
  const loaded = ref(false);

  async function load() {
    try {
      const cfg = await configApi.load();
      theme.value = (cfg.settings.theme as 'dark' | 'light' | 'auto') ?? 'dark';
      defaultView.value = (cfg.settings.defaultView as 'tree' | 'list') ?? 'tree';
    } finally {
      loaded.value = true;
    }
  }

  async function save() {
    if (!loaded.value) return;
    await invoke('settings_save', {
      args: { theme: theme.value, defaultView: defaultView.value },
    });
  }

  watch([theme, defaultView], () => {
    save();
  });

  return { theme, defaultView, loaded, load };
});
