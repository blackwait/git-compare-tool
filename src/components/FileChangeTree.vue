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
    :render-label="renderLabel"
    style="height: 100%"
    @update:selected-keys="onSelect"
  />
</template>

<script setup lang="ts">
import { computed, h } from 'vue';
import { NTree, type TreeOption } from 'naive-ui';
import type { FileChange } from '../types';

const props = defineProps<{ files: FileChange[]; selected?: string | null }>();
const emit = defineEmits<{ (e: 'select', path: string): void }>();

interface FileNode extends TreeOption {
  _file?: FileChange;
  _nodeType?: 'module' | 'dir' | 'file';
}

const treeData = computed<FileNode[]>(() => {
  const modules: Record<string, Record<string, FileChange[]>> = {};

  for (const f of props.files) {
    const segs = f.path.split('/');
    const moduleName = segs[0] || f.path;
    const dirPath = segs.length > 2 ? segs.slice(1, -1).join('/') : '';

    if (!modules[moduleName]) modules[moduleName] = {};
    if (!modules[moduleName][dirPath]) modules[moduleName][dirPath] = [];
    modules[moduleName][dirPath].push(f);
  }

  const result: FileNode[] = [];

  for (const [moduleName, dirs] of Object.entries(modules)) {
    const moduleChildren: FileNode[] = [];

    for (const [dirPath, files] of Object.entries(dirs)) {
      const fileNodes: FileNode[] = files.map((f) => ({
        key: f.path,
        label: f.path.split('/').pop() || f.path,
        isLeaf: true,
        _file: f,
        _nodeType: 'file' as const,
      }));

      if (dirPath) {
        moduleChildren.push({
          key: `${moduleName}/${dirPath}`,
          label: dirPath,
          children: fileNodes,
          _nodeType: 'dir',
        });
      } else {
        moduleChildren.push(...fileNodes);
      }
    }

    result.push({
      key: moduleName,
      label: moduleName,
      children: moduleChildren,
      _nodeType: 'module',
    });
  }

  return result;
});

function renderLabel({ option }: { option: TreeOption }) {
  const node = option as FileNode;

  // 模块节点
  if (node._nodeType === 'module') {
    return h('span', { style: 'display:inline-flex;align-items:center;gap:6px;font-weight:600' }, [
      h('span', { style: 'font-size:14px' }, '📦'),
      h('span', { style: 'color:#79c0ff' }, node.label as string),
    ]);
  }

  // 目录节点
  if (node._nodeType === 'dir') {
    return h('span', { style: 'display:inline-flex;align-items:center;gap:6px' }, [
      h('span', { style: 'font-size:13px' }, '📂'),
      h('span', { style: 'color:#e0e0e0' }, node.label as string),
    ]);
  }

  // 文件节点
  if (node._file) {
    const f = node._file;
    const fileName = f.path.split('/').pop() || f.path;
    const kindConfig: Record<string, { color: string; icon: string }> = {
      added: { color: '#56d364', icon: '✚' },
      modified: { color: '#d29922', icon: '✎' },
      deleted: { color: '#ff7b72', icon: '✖' },
      renamed: { color: '#79c0ff', icon: '➜' },
    };
    const cfg = kindConfig[f.kind] || { color: '#ffffff', icon: '•' };

    return h('span', { style: 'display:inline-flex;align-items:center;gap:6px;font-size:12px' }, [
      h('span', { style: `color:${cfg.color};font-size:12px;font-weight:700` }, cfg.icon),
      h('span', { style: `color:${cfg.color}` }, fileName),
      h('span', { style: 'color:#56d364;font-size:11px;font-weight:600;margin-left:4px' }, `+${f.additions}`),
      h('span', { style: 'color:#ff7b72;font-size:11px;font-weight:600' }, `-${f.deletions}`),
    ]);
  }

  return h('span', {}, node.label as string);
}

function onSelect(keys: Array<string | number>) {
  const k = keys[0];
  if (typeof k !== 'string') return;
  const hit = props.files.find((f) => f.path === k);
  if (hit) emit('select', hit.path);
}
</script>
