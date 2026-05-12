<template>
  <n-menu mode="horizontal" :options="options" @update:value="onClick" />
</template>

<script setup lang="ts">
import { NMenu, type MenuOption } from 'naive-ui';
import { useRouter } from 'vue-router';
import { useWorkspacesStore } from '../stores/workspaces';

const router = useRouter();
const store = useWorkspacesStore();

const options: MenuOption[] = [
  {
    label: '仓库',
    key: 'repo',
    children: [
      { label: '+ 新建工作区', key: 'new' },
      { label: '刷新列表', key: 'reload' },
    ],
  },
  { label: '设置', key: 'settings' },
  { label: '欢迎页', key: 'welcome' },
];

function onClick(key: string) {
  if (key === 'new') window.dispatchEvent(new Event('gct:open-create'));
  if (key === 'reload') store.reload();
  if (key === 'settings') router.push({ name: 'settings' });
  if (key === 'welcome') router.push({ name: 'welcome' });
}
</script>
