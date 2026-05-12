<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <div class="layout">
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
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NNotificationProvider,
  darkTheme,
} from 'naive-ui';
import WorkspaceSidebar from './components/WorkspaceSidebar.vue';
</script>

<style>
html, body, #app {
  height: 100%;
  margin: 0;
  padding: 0;
}
.layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
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
