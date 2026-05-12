<template>
  <div>
    <div
      class="tree-row"
      :class="{ selected: node.isLeaf && node.key === selected, leaf: node.isLeaf }"
      :style="{ paddingLeft: (depth * 16 + 12) + 'px' }"
      @click="onClick"
      @contextmenu.prevent="onContextMenu"
    >
      <span v-if="!node.isLeaf" class="expand-icon">{{ expanded ? '▾' : '▸' }}</span>
      <span v-if="!node.isLeaf" class="node-icon folder">📁</span>
      <span v-else class="node-icon branch">🔀</span>
      <span class="node-label" :class="{ head: node.isHead }">
        {{ node.isHead ? '★ ' : '' }}{{ node.label }}
      </span>
    </div>
    <!-- 右键菜单 teleport 到 body 避免定位偏移 -->
    <Teleport to="body">
      <div
        v-if="showMenu && node.isLeaf"
        class="context-menu"
        :style="{ left: menuX + 'px', top: menuY + 'px' }"
      >
        <div class="menu-item" @click="onCopyName">📋 复制名称</div>
        <div class="menu-item" @click="onUpdateBranch">⬇️ 更新代码</div>
      </div>
    </Teleport>
    <div v-if="!node.isLeaf && expanded">
      <BranchTreeNode
        v-for="child in node.children"
        :key="child.key"
        :node="child"
        :depth="depth + 1"
        :selected="selected"
        @select="(v) => emit('select', v)"
        @copy-name="(v) => emit('copy-name', v)"
        @update-branch="(v) => emit('update-branch', v)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import type { BranchNode } from './BranchPicker.vue';

const props = defineProps<{
  node: BranchNode;
  depth: number;
  selected: string | null;
}>();
const emit = defineEmits<{
  (e: 'select', v: string): void;
  (e: 'copy-name', v: string): void;
  (e: 'update-branch', v: string): void;
}>();

const expanded = ref(true);
const showMenu = ref(false);
const menuX = ref(0);
const menuY = ref(0);

function onClick() {
  if (props.node.isLeaf) {
    emit('select', props.node.key);
  } else {
    expanded.value = !expanded.value;
  }
}

function onContextMenu(e: MouseEvent) {
  if (!props.node.isLeaf) return;
  showMenu.value = true;
  menuX.value = e.clientX;
  menuY.value = e.clientY;
}

function onCopyName() {
  showMenu.value = false;
  emit('copy-name', props.node.key);
}

function onUpdateBranch() {
  showMenu.value = false;
  emit('update-branch', props.node.key);
}

function closeMenu() {
  showMenu.value = false;
}

onMounted(() => document.addEventListener('click', closeMenu));
onUnmounted(() => document.removeEventListener('click', closeMenu));
</script>

<style scoped>
.tree-row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  cursor: pointer;
  font-size: 12px;
  color: #e0e0e0;
  user-select: none;
  white-space: nowrap;
}
.tree-row:hover {
  background: rgba(64, 150, 255, 0.12);
}
.tree-row.selected {
  background: rgba(64, 150, 255, 0.25);
  color: #ffffff;
}
.expand-icon {
  width: 12px;
  font-size: 10px;
  color: #888;
  text-align: center;
}
.node-icon {
  font-size: 13px;
}
.node-icon.folder {
  font-size: 12px;
}
.node-icon.branch {
  font-size: 12px;
}
.node-label {
  flex: 1;
  white-space: nowrap;
}
.node-label.head {
  color: #f0c674;
  font-weight: 600;
}
</style>

<style>
.context-menu {
  position: fixed;
  z-index: 9999;
  background: #2d2d2d;
  border: 1px solid #444;
  border-radius: 4px;
  padding: 4px 0;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  min-width: 140px;
}
.menu-item {
  padding: 6px 14px;
  font-size: 12px;
  color: #e0e0e0;
  cursor: pointer;
  white-space: nowrap;
}
.menu-item:hover {
  background: rgba(64, 150, 255, 0.2);
  color: #ffffff;
}
</style>
