<template>
  <div class="file-list">
    <div
      v-for="f in files"
      :key="f.path"
      class="row"
      :class="{ active: selected === f.path }"
      @click="emit('select', f.path)"
    >
      <span class="badge" :class="f.kind">{{ badge(f.kind) }}</span>
      <span class="path">{{ f.oldPath ? `${f.oldPath} → ${f.path}` : f.path }}</span>
      <span class="stat">
        <span class="add">+{{ f.additions }}</span>
        <span class="del">-{{ f.deletions }}</span>
      </span>
    </div>
    <n-empty v-if="!files.length" description="没有差异" style="padding: 20px" />
  </div>
</template>

<script setup lang="ts">
import { NEmpty } from 'naive-ui';
import type { FileChange } from '../types';

defineProps<{ files: FileChange[]; selected?: string | null }>();
const emit = defineEmits<{ (e: 'select', path: string): void }>();

function badge(k: FileChange['kind']): string {
  return { added: 'A', modified: 'M', deleted: 'D', renamed: 'R' }[k];
}
</script>

<style scoped>
.file-list {
  height: 100%;
  overflow: auto;
  font-family: 'Fira Code', monospace;
  font-size: 12px;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  cursor: pointer;
  border-bottom: 1px solid #1f1f1f;
}
.row:hover,
.row.active {
  background: rgba(32, 128, 255, 0.08);
}
.badge {
  width: 18px;
  text-align: center;
  border-radius: 2px;
  font-weight: bold;
}
.badge.added { background: #234d20; color: #6acf7c; }
.badge.modified { background: #4d4120; color: #e8c068; }
.badge.deleted { background: #4d2020; color: #ff6a6a; }
.badge.renamed { background: #20344d; color: #6aa9ff; }
.path {
  flex: 1;
  word-break: break-all;
}
.stat .add { color: #6acf7c; }
.stat .del { color: #ff6a6a; margin-left: 6px; }
</style>
