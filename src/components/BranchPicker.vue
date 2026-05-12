<template>
  <n-select
    :value="modelValue"
    :options="options"
    filterable
    :loading="!!loading"
    placeholder="选择分支"
    style="min-width: 240px"
    @update:value="emit('update:modelValue', $event)"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NSelect, type SelectGroupOption } from 'naive-ui';
import type { Branch } from '../types';

const props = defineProps<{
  branches: Branch[];
  modelValue: string | null;
  loading?: boolean;
}>();
const emit = defineEmits<{ (e: 'update:modelValue', v: string): void }>();

const options = computed<SelectGroupOption[]>(() => {
  const local = props.branches
    .filter((b) => b.kind === 'local')
    .map((b) => ({
      label: (b.isHead ? '★ ' : '') + b.name,
      value: b.name,
    }));
  const remote = props.branches
    .filter((b) => b.kind === 'remote')
    .map((b) => ({ label: b.name, value: b.name }));
  return [
    { type: 'group', label: '本地', key: 'local', children: local },
    { type: 'group', label: '远程', key: 'remote', children: remote },
  ];
});
</script>
