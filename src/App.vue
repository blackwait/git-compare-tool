<template>
  <n-config-provider :theme="effectiveTheme">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <div class="layout">
            <div class="top">
              <AppMenu />
            </div>
            <div class="body">
              <WorkspaceSidebar />
              <main class="main">
                <router-view />
              </main>
            </div>
          </div>
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  darkTheme,
} from 'naive-ui';
import AppMenu from './components/AppMenu.vue';
import WorkspaceSidebar from './components/WorkspaceSidebar.vue';
import { useSettingsStore } from './stores/settings';
import { useDiffStore } from './stores/diff';

const s = useSettingsStore();
const diff = useDiffStore();

const effectiveTheme = computed(() => (s.theme === 'light' ? null : darkTheme));

onMounted(async () => {
  await s.load();
  diff.viewMode = s.defaultView;
});
</script>

<style>
html,
body,
#app {
  height: 100%;
  margin: 0;
  padding: 0;
}
.layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
.top {
  border-bottom: 1px solid #2a2a2a;
}
.body {
  flex: 1;
  display: flex;
  overflow: hidden;
}
.main {
  flex: 1;
  overflow: auto;
  padding: 16px;
}
</style>
