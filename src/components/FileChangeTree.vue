<template>
  <n-tree
    block-line
    block-node
    :data="treeData"
    key-field="key"
    label-field="label"
    :selected-keys="selected ? [selected] : []"
    :default-expand-all="true"
    :virtual-scroll="true"
    style="height: 100%"
    @update:selected-keys="onSelect"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { NTree, type TreeOption } from 'naive-ui';
import type { FileChange } from '../types';

const props = defineProps<{ files: FileChange[]; selected?: string | null }>();
const emit = defineEmits<{ (e: 'select', path: string): void }>();

interface TmpNode {
  key: string;
  label: string;
  leaf?: boolean;
  children: Record<string, TmpNode>;
}

const treeData = computed<TreeOption[]>(() => {
  const root: TmpNode = { key: '', label: '', children: {} };
  for (const f of props.files) {
    const segs = f.path.split('/');
    let cur = root;
    segs.forEach((s, i) => {
      if (i === segs.length - 1) {
        cur.children[s] = {
          key: f.path,
          label: `${s}  +${f.additions} -${f.deletions}`,
          leaf: true,
          children: {},
        };
      } else {
        const pathKey = segs.slice(0, i + 1).join('/');
        if (!cur.children[s]) {
          cur.children[s] = { key: pathKey, label: s, children: {} };
        }
        cur = cur.children[s];
      }
    });
  }
  const build = (node: TmpNode): TreeOption[] =>
    Object.entries(node.children).map(([, child]) => {
      if (child.leaf) {
        return { key: child.key, label: child.label, isLeaf: true };
      }
      return { key: child.key, label: child.label, children: build(child) };
    });
  return build(root);
});

function onSelect(keys: Array<string | number>) {
  const k = keys[0];
  if (typeof k !== 'string') return;
  const hit = props.files.find((f) => f.path === k);
  if (hit) emit('select', hit.path);
}
</script>
