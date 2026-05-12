<template>
  <n-tabs v-model:value="tab" type="line" style="height: 100%; padding-left: 12px">
    <n-tab-pane name="diff" tab="Diff">
      <div v-if="fileDiff?.isBinary" class="tip">二进制文件，无法预览</div>
      <div v-else-if="!fileDiff || !fileDiff.hunks.length" class="tip">未选择文件或无差异</div>
      <div v-else class="diff">
        <template v-for="(h, hi) in fileDiff.hunks" :key="hi">
          <!-- 两个 hunk 之间的折叠区域 -->
          <div v-if="hi > 0" class="gap-section">
            <div class="gap-head" @click="toggleGap(hi)">
              <span class="gap-icon">{{ expandedGaps[hi] ? '▾' : '▸' }}</span>
              <span class="gap-dots">···</span>
              <span class="gap-info">{{ getGapLineCount(hi) }} 行未变更代码</span>
              <span class="gap-action">{{ expandedGaps[hi] ? '收起' : '展开' }}</span>
            </div>
            <div v-if="expandedGaps[hi]" class="gap-lines">
              <div v-if="gapLoading[hi]" class="tip">加载中...</div>
              <template v-else-if="gapLines[hi]">
                <div v-for="(l, li) in gapLines[hi]" :key="li" class="line context">
                  <span class="ln">{{ l.oldLineNo }}</span>
                  <span class="ln">{{ l.newLineNo }}</span>
                  <pre class="content">  {{ l.content }}</pre>
                </div>
              </template>
            </div>
          </div>
          <!-- hunk header + lines -->
          <div class="hunk">
            <div class="hunk-head">
              <span class="hunk-icon">§</span>
              旧 第{{ h.oldStart }}行 共{{ h.oldLines }}行
              <span class="hunk-sep">→</span>
              新 第{{ h.newStart }}行 共{{ h.newLines }}行
              <span v-if="h.oldLines > h.newLines" class="hunk-tag del-tag">-{{ h.oldLines - h.newLines }}</span>
              <span v-else-if="h.newLines > h.oldLines" class="hunk-tag add-tag">+{{ h.newLines - h.oldLines }}</span>
            </div>
            <div v-for="(l, li) in h.lines" :key="li" class="line" :class="l.kind">
              <span class="ln">{{ l.oldLineNo ?? '' }}</span>
              <span class="ln">{{ l.newLineNo ?? '' }}</span>
              <pre class="content">{{ prefix(l.kind) + l.content }}</pre>
            </div>
          </div>
        </template>
        <div v-if="fileDiff.truncated" class="tip">结果过大，已截断</div>
      </div>
    </n-tab-pane>
    <n-tab-pane name="source" tab="源文件 (目标分支)">
      <div v-if="!path" class="tip">未选择文件</div>
      <template v-else>
        <pre v-if="source !== null" class="source hljs" v-html="highlighted" />
        <div v-else-if="loadingSrc" class="tip">加载中...</div>
      </template>
    </n-tab-pane>
  </n-tabs>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue';
import { NTabs, NTabPane } from 'naive-ui';
import hljs from 'highlight.js';
import 'highlight.js/styles/atom-one-dark.css';
import type { FileDiff, DiffHunk } from '../types';
import { gitApi } from '../api/git';

interface GapLine {
  oldLineNo: number;
  newLineNo: number;
  content: string;
}

const props = defineProps<{
  repoId: string | null;
  targetRef: string | null;
  path: string | null;
  fileDiff: FileDiff | null;
}>();

const tab = ref<'diff' | 'source'>('diff');
const source = ref<string | null>(null);
const loadingSrc = ref(false);

// 展开/折叠状态
const expandedGaps = reactive<Record<number, boolean>>({});
const gapLines = reactive<Record<number, GapLine[]>>({});
const gapLoading = reactive<Record<number, boolean>>({});

function prefix(k: string): string {
  return k === 'add' ? '+' : k === 'del' ? '-' : ' ';
}

/** 计算两个 hunk 之间省略的行数 */
function getGapLineCount(hunkIndex: number): number {
  if (!props.fileDiff || hunkIndex <= 0) return 0;
  const prev = props.fileDiff.hunks[hunkIndex - 1];
  const curr = props.fileDiff.hunks[hunkIndex];
  const prevEnd = prev.newStart + prev.newLines - 1;
  const currStart = curr.newStart;
  return Math.max(0, currStart - prevEnd - 1);
}

/** 获取 gap 区域的起止行号（基于 new/target 分支） */
function getGapRange(hunkIndex: number): { startLine: number; endLine: number } {
  if (!props.fileDiff || hunkIndex <= 0) return { startLine: 0, endLine: 0 };
  const prev = props.fileDiff.hunks[hunkIndex - 1];
  const curr = props.fileDiff.hunks[hunkIndex];
  const startLine = prev.newStart + prev.newLines;
  const endLine = curr.newStart - 1;
  return { startLine, endLine };
}

/** 同样计算 old 侧的行号范围 */
function getGapRangeOld(hunkIndex: number): { startLine: number; endLine: number } {
  if (!props.fileDiff || hunkIndex <= 0) return { startLine: 0, endLine: 0 };
  const prev = props.fileDiff.hunks[hunkIndex - 1];
  const curr = props.fileDiff.hunks[hunkIndex];
  const startLine = prev.oldStart + prev.oldLines;
  const endLine = curr.oldStart - 1;
  return { startLine, endLine };
}

async function toggleGap(hunkIndex: number) {
  if (expandedGaps[hunkIndex]) {
    expandedGaps[hunkIndex] = false;
    return;
  }
  expandedGaps[hunkIndex] = true;
  // 已加载过
  if (gapLines[hunkIndex]) return;
  // 加载源文件内容
  if (!props.repoId || !props.targetRef || !props.path) return;
  gapLoading[hunkIndex] = true;
  try {
    if (source.value === null) {
      source.value = await gitApi.fileContent(props.repoId, props.targetRef, props.path);
    }
    const { startLine, endLine } = getGapRange(hunkIndex);
    const oldRange = getGapRangeOld(hunkIndex);
    const lines = source.value!.split('\n');
    const result: GapLine[] = [];
    for (let i = startLine; i <= endLine; i++) {
      const oldLineNo = oldRange.startLine + (i - startLine);
      result.push({
        oldLineNo,
        newLineNo: i,
        content: lines[i - 1] ?? '',
      });
    }
    gapLines[hunkIndex] = result;
  } finally {
    gapLoading[hunkIndex] = false;
  }
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

// 切换文件时重置状态
watch(
  () => props.path,
  () => {
    source.value = null;
    tab.value = 'diff';
    Object.keys(expandedGaps).forEach((k) => delete expandedGaps[+k]);
    Object.keys(gapLines).forEach((k) => delete gapLines[+k]);
    Object.keys(gapLoading).forEach((k) => delete gapLoading[+k]);
  }
);

watch(tab, (v) => {
  if (v === 'source' && source.value === null) {
    loadSource();
  }
});
</script>

<style scoped>
.tip {
  padding: 12px;
  color: #a0a0a0;
  font-size: 12px;
}
.diff {
  font-family: 'JetBrains Mono', 'Microsoft YaHei', monospace;
  font-size: 12px;
  color: #ffffff;
}
.hunk-head {
  background: #1c3050;
  color: #9fc4ff;
  padding: 5px 16px;
  border-top: 1px solid #2a2a2a;
  border-bottom: 1px solid #2a2a2a;
  font-weight: 600;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
}
.hunk-icon {
  color: #79c0ff;
  font-size: 14px;
  font-weight: 700;
}
.hunk-sep {
  color: #6e7681;
  margin: 0 2px;
}
.hunk-tag {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 3px;
  font-weight: 700;
  margin-left: 6px;
}
.hunk-tag.add-tag {
  background: rgba(46, 160, 67, 0.25);
  color: #56d364;
}
.hunk-tag.del-tag {
  background: rgba(218, 54, 51, 0.25);
  color: #ff7b72;
}
.gap-section {
  border-top: 1px solid #2a2a2a;
  border-bottom: 1px solid #2a2a2a;
}
.gap-head {
  background: #1a2332;
  color: #6e8fa8;
  padding: 4px 16px;
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  user-select: none;
}
.gap-head:hover {
  background: #1e2a3a;
  color: #9fc4ff;
}
.gap-icon {
  font-size: 10px;
  width: 12px;
}
.gap-dots {
  color: #4a6a8a;
  letter-spacing: 2px;
}
.gap-info {
  flex: 1;
  font-size: 11px;
}
.gap-action {
  font-size: 11px;
  color: #4a9eff;
  opacity: 0;
  transition: opacity 0.15s;
}
.gap-head:hover .gap-action {
  opacity: 1;
}
.line {
  display: grid;
  grid-template-columns: 50px 50px 1fr;
  line-height: 20px;
}
.line .ln {
  color: #8a8a8a;
  text-align: right;
  padding: 2px 8px;
  background: #1a1a1a;
  user-select: none;
  border-right: 1px solid #2a2a2a;
}
.line .content {
  margin: 0;
  padding: 2px 10px;
  white-space: pre-wrap;
  font-family: inherit;
  color: #ffffff;
}
.line.context .content { color: #e8e8e8; }
.line.add {
  background: rgba(46, 160, 67, 0.18);
}
.line.add .content { color: #aff5b4; }
.line.add .ln { background: #14291a; color: #7ee787; }
.line.del {
  background: rgba(218, 54, 51, 0.2);
}
.line.del .content { color: #ffb3ae; }
.line.del .ln { background: #2a1515; color: #ff7b72; }
.source {
  padding: 10px 14px;
  font-size: 12px;
  white-space: pre-wrap;
  margin: 0;
  color: #ffffff;
  background: #1a1a1a;
}
</style>
