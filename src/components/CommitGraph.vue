<template>
  <div class="graph-wrap">
    <div class="title">提交历史 ({{ commits.length }} 个 commit)</div>
    <div v-if="!commits.length" class="hint">无提交</div>
    <svg v-else :width="800" :height="commits.length * 26 + 10" class="graph">
      <g
        v-for="(c, i) in commits"
        :key="c.hash"
        :transform="`translate(10, ${i * 26 + 14})`"
      >
        <line
          v-if="i < commits.length - 1"
          x1="0"
          y1="0"
          x2="0"
          y2="26"
          stroke="#555"
        />
        <circle r="5" :fill="c.parents.length > 1 ? '#e8c068' : '#6aa9ff'" />
        <text x="16" y="4" fill="#aaa" font-size="11" font-family="Fira Code, monospace">
          {{ c.shortHash }}
        </text>
        <text x="100" y="4" fill="#ddd" font-size="12">
          {{ c.message.length > 80 ? c.message.slice(0, 80) + '…' : c.message }}
        </text>
        <text x="720" y="4" fill="#888" font-size="11">{{ c.author }}</text>
      </g>
    </svg>
  </div>
</template>

<script setup lang="ts">
import type { Commit } from '../types';
defineProps<{ commits: Commit[] }>();
</script>

<style scoped>
.graph-wrap {
  padding: 8px 0;
  overflow: auto;
  max-height: 240px;
  border-top: 1px solid #2a2a2a;
}
.title {
  padding: 0 12px 4px;
  color: #aaa;
  font-size: 12px;
}
.hint {
  padding: 8px 12px;
  color: #888;
  font-size: 12px;
}
</style>
