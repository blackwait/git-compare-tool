<template>
  <n-popover trigger="click" placement="bottom-start" :show="showPopover" @update:show="showPopover = $event" style="padding: 0">
    <template #trigger>
      <div class="branch-trigger" @click="showPopover = !showPopover">
        <span class="branch-icon">🔀</span>
        <span class="branch-text">{{ modelValue || '选择分支' }}</span>
        <span class="branch-arrow">▾</span>
      </div>
    </template>
    <div class="branch-panel">
      <div class="branch-search">
        <input
          v-model="searchText"
          class="search-input"
          placeholder="搜索分支..."
          @input="onSearch"
        />
      </div>
      <div class="branch-tree">
        <div class="group-header">📁 本地</div>
        <div class="tree-content">
          <template v-for="node in localTree" :key="node.key">
            <BranchTreeNode
              :node="node"
              :depth="0"
              :selected="modelValue"
              @select="onSelect"
              @copy-name="(v) => emit('copy-name', v)"
              @update-branch="(v) => emit('update-branch', v)"
            />
          </template>
        </div>
        <div class="group-header">📁 远程</div>
        <div class="tree-content">
          <template v-for="node in remoteTree" :key="node.key">
            <BranchTreeNode
              :node="node"
              :depth="0"
              :selected="modelValue"
              @select="onSelect"
              @copy-name="(v) => emit('copy-name', v)"
              @update-branch="(v) => emit('update-branch', v)"
            />
          </template>
        </div>
      </div>
    </div>
  </n-popover>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { NPopover } from 'naive-ui';
import type { Branch } from '../types';
import BranchTreeNode from './BranchTreeNode.vue';

export interface BranchNode {
  key: string;
  label: string;
  isLeaf: boolean;
  isHead?: boolean;
  children?: BranchNode[];
}

const props = defineProps<{
  branches: Branch[];
  modelValue: string | null;
  loading?: boolean;
}>();
const emit = defineEmits<{
  (e: 'update:modelValue', v: string): void;
  (e: 'copy-name', v: string): void;
  (e: 'update-branch', v: string): void;
}>();

const showPopover = ref(false);
const searchText = ref('');

function buildTree(branches: Branch[], kind: 'local' | 'remote'): BranchNode[] {
  const filtered = branches.filter((b) => b.kind === kind);
  const searched = searchText.value
    ? filtered.filter((b) => b.name.toLowerCase().includes(searchText.value.toLowerCase()))
    : filtered;

  const root: Record<string, any> = {};

  for (const b of searched) {
    const parts = b.name.split('/');
    let current = root;
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      if (i === parts.length - 1) {
        current[part] = { __leaf: true, __name: b.name, __isHead: b.isHead };
      } else {
        if (!current[part] || current[part].__leaf) {
          current[part] = current[part]?.__leaf ? { ...current[part] } : {};
        }
        current = current[part];
      }
    }
  }

  function toNodes(obj: Record<string, any>, prefix: string): BranchNode[] {
    const nodes: BranchNode[] = [];
    const entries = Object.entries(obj).filter(([k]) => !k.startsWith('__'));

    for (const [key, val] of entries) {
      const fullKey = prefix ? `${prefix}/${key}` : key;
      if (val.__leaf) {
        nodes.push({
          key: val.__name,
          label: key,
          isLeaf: true,
          isHead: val.__isHead,
        });
      } else {
        const children = toNodes(val, fullKey);
        nodes.push({
          key: fullKey,
          label: key,
          isLeaf: false,
          children,
        });
      }
    }
    return nodes;
  }

  return toNodes(root, '');
}

const localTree = computed(() => buildTree(props.branches, 'local'));
const remoteTree = computed(() => buildTree(props.branches, 'remote'));

function onSelect(branchName: string) {
  emit('update:modelValue', branchName);
  showPopover.value = false;
}

function onSearch() {}
</script>

<style scoped>
.branch-trigger {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  cursor: pointer;
  min-width: 220px;
  color: #ffffff;
  font-size: 13px;
}
.branch-trigger:hover {
  border-color: #4a9eff;
}
.branch-icon {
  font-size: 14px;
}
.branch-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.branch-arrow {
  color: #888;
  font-size: 11px;
}
.branch-panel {
  min-width: 340px;
  max-height: 400px;
  display: flex;
  flex-direction: column;
  background: #252526;
  border-radius: 4px;
  overflow: visible;
}
.branch-search {
  padding: 8px;
  border-bottom: 1px solid #3a3a3a;
}
.search-input {
  width: 100%;
  padding: 6px 10px;
  background: #1e1e1e;
  border: 1px solid #3a3a3a;
  border-radius: 3px;
  color: #ffffff;
  font-size: 12px;
  outline: none;
  box-sizing: border-box;
}
.search-input:focus {
  border-color: #4a9eff;
}
.search-input::placeholder {
  color: #888;
}
.branch-tree {
  flex: 1;
  overflow-y: auto;
  overflow-x: visible;
  padding: 4px 0;
}
.group-header {
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  color: #9d9d9d;
  user-select: none;
}
.tree-content {
  padding-bottom: 4px;
}
</style>
