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
  font-family: 'JetBrains Mono', 'Microsoft YaHei', monospace;
  font-size: 12px;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  cursor: pointer;
  border-bottom: 1px solid #2b2b2b;
  color: #ffffff;
}
.row:hover { background: rgba(64, 150, 255, 0.12); }
.row.active { background: rgba(64, 150, 255, 0.22); color: #ffffff; }
.badge {
  min-width: 20px;
  height: 18px;
  line-height: 18px;
  text-align: center;
  border-radius: 3px;
  font-weight: 700;
  font-size: 11px;
  color: #ffffff;
  padding: 0 4px;
}
.badge.added    { background: #2ea043; }
.badge.modified { background: #d29922; }
.badge.deleted  { background: #da3633; }
.badge.renamed  { background: #1f6feb; }
.path { flex: 1; word-break: break-all; color: #ffffff; }
.stat .add { color: #56d364; font-weight: 600; }
.stat .del { color: #ff7b72; font-weight: 600; margin-left: 6px; }
</style>
