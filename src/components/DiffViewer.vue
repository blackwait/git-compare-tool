<template>
  <n-tabs v-model:value="tab" type="line">
    <n-tab-pane name="diff" tab="Diff">
      <div v-if="fileDiff?.isBinary" class="tip">二进制文件，无法预览</div>
      <div v-else-if="!fileDiff || !fileDiff.hunks.length" class="tip">未选择文件或无差异</div>
      <div v-else class="diff">
        <div v-for="(h, hi) in fileDiff.hunks" :key="hi" class="hunk">
          <div class="hunk-head">
            @@ -{{ h.oldStart }},{{ h.oldLines }} +{{ h.newStart }},{{ h.newLines }} @@
          </div>
          <div v-for="(l, li) in h.lines" :key="li" class="line" :class="l.kind">
            <span class="ln">{{ l.oldLineNo ?? '' }}</span>
            <span class="ln">{{ l.newLineNo ?? '' }}</span>
            <pre class="content">{{ prefix(l.kind) + l.content }}</pre>
          </div>
        </div>
        <div v-if="fileDiff.truncated" class="tip">结果过大，已截断</div>
      </div>
    </n-tab-pane>
    <n-tab-pane name="source" tab="源文件 (目标分支)">
      <div v-if="!path" class="tip">未选择文件</div>
      <template v-else>
        <n-space style="padding: 8px 12px" :size="8">
          <n-button size="small" :loading="loadingSrc" @click="loadSource">
            {{ source === null ? '加载源文件' : '重新加载' }}
          </n-button>
          <span v-if="source !== null" style="color: #888; font-size: 12px; align-self: center">
            {{ targetRef }} : {{ path }}
          </span>
        </n-space>
        <pre v-if="source !== null" class="source hljs" v-html="highlighted" />
      </template>
    </n-tab-pane>
  </n-tabs>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { NTabs, NTabPane, NButton, NSpace } from 'naive-ui';
import hljs from 'highlight.js';
import 'highlight.js/styles/atom-one-dark.css';
import type { FileDiff } from '../types';
import { gitApi } from '../api/git';

const props = defineProps<{
  repoId: string | null;
  targetRef: string | null;
  path: string | null;
  fileDiff: FileDiff | null;
}>();

const tab = ref<'diff' | 'source'>('diff');
const source = ref<string | null>(null);
const loadingSrc = ref(false);

function prefix(k: string): string {
  return k === 'add' ? '+' : k === 'del' ? '-' : ' ';
}

const highlighted = computed(() => {
  if (source.value === null) return '';
  try {
    return hljs.highlightAuto(source.value).value;
  } catch {
    return source.value;
  }
});

async function loadSource() {
  if (!props.repoId || !props.targetRef || !props.path) return;
  loadingSrc.value = true;
  try {
    source.value = await gitApi.fileContent(props.repoId, props.targetRef, props.path);
  } finally {
    loadingSrc.value = false;
  }
}

watch(
  () => props.path,
  () => {
    source.value = null;
    tab.value = 'diff';
  }
);
</script>

<style scoped>
.tip {
  padding: 12px;
  color: #888;
  font-size: 12px;
}
.diff {
  font-family: 'Fira Code', monospace;
  font-size: 12px;
}
.hunk-head {
  background: #1a2332;
  color: #9ec5ff;
  padding: 2px 8px;
}
.line {
  display: grid;
  grid-template-columns: 50px 50px 1fr;
}
.line .ln {
  color: #666;
  text-align: right;
  padding: 0 8px;
  background: #151515;
  user-select: none;
}
.line .content {
  margin: 0;
  white-space: pre-wrap;
  font-family: inherit;
}
.line.add {
  background: rgba(80, 200, 120, 0.08);
}
.line.add .content {
  color: #b0e6b0;
}
.line.del {
  background: rgba(255, 80, 80, 0.08);
}
.line.del .content {
  color: #e0a0a0;
}
.source {
  padding: 8px 12px;
  font-size: 12px;
  white-space: pre-wrap;
  margin: 0;
}
</style>
