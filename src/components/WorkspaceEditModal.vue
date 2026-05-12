<template>
  <n-modal
    :show="show"
    preset="card"
    :title="workspace ? '编辑仓库' : '新建仓库'"
    style="width: 520px"
    @update:show="emit('update:show', $event)"
  >
    <n-form :model="form" label-placement="left" label-width="auto">
      <n-form-item label="名称">
        <n-input v-model:value="form.name" placeholder="仓库显示名" />
      </n-form-item>
      <n-form-item label="路径">
        <n-input-group>
          <n-input v-model:value="form.path" placeholder="本地仓库绝对路径" />
          <n-button @click="onPick">选择...</n-button>
        </n-input-group>
      </n-form-item>
      <n-form-item label="远端名">
        <n-input v-model:value="form.remoteName" placeholder="origin" />
      </n-form-item>
    </n-form>
    <template #footer>
      <n-space justify="end">
        <n-button @click="emit('update:show', false)">取消</n-button>
        <n-button type="primary" :loading="saving" @click="onSave">保存</n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import {
  NModal,
  NForm,
  NFormItem,
  NInput,
  NInputGroup,
  NButton,
  NSpace,
  useMessage,
} from 'naive-ui';
import type { Workspace } from '../types';
import { configApi } from '../api/config';
import { gitApi } from '../api/git';

const props = defineProps<{ show: boolean; workspace?: Workspace | null }>();
const emit = defineEmits<{
  (e: 'update:show', v: boolean): void;
  (e: 'saved', w: Workspace): void;
}>();

const message = useMessage();
const saving = ref(false);
const form = ref({ name: '', path: '', remoteName: 'origin' });

watch(
  () => props.show,
  (v) => {
    if (v) {
      form.value = props.workspace
        ? {
            name: props.workspace.name,
            path: props.workspace.path,
            remoteName: props.workspace.remoteName ?? 'origin',
          }
        : { name: '', path: '', remoteName: 'origin' };
    }
  }
);

async function onPick() {
  const p = await configApi.pickDir();
  if (p) form.value.path = p;
}

async function onSave() {
  if (!form.value.name.trim() || !form.value.path.trim()) {
    message.error('名称与路径必填');
    return;
  }
  saving.value = true;
  try {
    const v = await gitApi.validateRepo(form.value.path);
    if (!v.valid) {
      message.error('所选路径不是 git 仓库');
      return;
    }
    const saved = props.workspace
      ? await configApi.updateWorkspace({ id: props.workspace.id, patch: form.value })
      : await configApi.createWorkspace(form.value);
    emit('saved', saved);
    emit('update:show', false);
    message.success('已保存');
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    message.error(msg);
  } finally {
    saving.value = false;
  }
}
</script>
