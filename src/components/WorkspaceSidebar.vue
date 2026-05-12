<template>
  <div class="sidebar">
    <n-space vertical size="small">
      <n-button block type="primary" @click="openCreate">+ 新建工作区</n-button>
      <n-list hoverable clickable bordered>
        <n-list-item
          v-for="w in store.list"
          :key="w.id"
          :class="{ active: route.params.id === w.id }"
          @click="select(w.id)"
        >
          <n-thing :title="w.name" content-indented>
            <template #description>
              <span class="path">{{ w.path }}</span>
            </template>
            <template #header-extra>
              <n-space :size="4">
                <n-button text size="tiny" @click.stop="openEdit(w)">编辑</n-button>
                <n-button text size="tiny" type="error" @click.stop="onDelete(w)">删除</n-button>
              </n-space>
            </template>
          </n-thing>
        </n-list-item>
        <n-empty
          v-if="!store.loading && !store.list.length"
          description="还没有工作区"
          style="padding: 16px"
        />
      </n-list>
    </n-space>

    <WorkspaceEditModal v-model:show="modal" :workspace="editing" @saved="onSaved" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  NSpace,
  NButton,
  NList,
  NListItem,
  NThing,
  NEmpty,
  useDialog,
  useMessage,
} from 'naive-ui';
import type { Workspace } from '../types';
import { useWorkspacesStore } from '../stores/workspaces';
import { configApi } from '../api/config';
import WorkspaceEditModal from './WorkspaceEditModal.vue';

const store = useWorkspacesStore();
const route = useRoute();
const router = useRouter();
const dialog = useDialog();
const message = useMessage();
const modal = ref(false);
const editing = ref<Workspace | null>(null);

onMounted(() => store.reload());

const openFromMenu = () => openCreate();
onMounted(() => window.addEventListener('gct:open-create', openFromMenu));
onUnmounted(() => window.removeEventListener('gct:open-create', openFromMenu));

function openCreate() {
  editing.value = null;
  modal.value = true;
}
function openEdit(w: Workspace) {
  editing.value = w;
  modal.value = true;
}
async function onSaved() {
  await store.reload();
}

function select(id: string) {
  router.push({ name: 'repo', params: { id } });
}

function onDelete(w: Workspace) {
  dialog.warning({
    title: '删除确认',
    content: `删除工作区 "${w.name}"？不会影响本地仓库文件。`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      await configApi.deleteWorkspace(w.id);
      await store.reload();
      message.success('已删除');
      if (route.params.id === w.id) router.push({ name: 'welcome' });
    },
  });
}
</script>

<style scoped>
.sidebar {
  width: 280px;
  padding: 12px;
  border-right: 1px solid #2a2a2a;
  height: 100%;
  overflow-y: auto;
  box-sizing: border-box;
}
.active {
  background: rgba(32, 128, 255, 0.1);
}
.path {
  font-size: 12px;
  color: #888;
  word-break: break-all;
}
</style>
