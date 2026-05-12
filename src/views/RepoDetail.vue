<template>
  <div v-if="repo.current" class="repo-detail">
    <n-space align="center" :size="12" style="flex-wrap: wrap">
      <h2 style="margin: 0">{{ repo.current.name }}</h2>
      <BranchPicker :branches="repo.branches" v-model="repo.base" :loading="loading" />
      <span>⇄</span>
      <BranchPicker :branches="repo.branches" v-model="repo.target" :loading="loading" />
      <n-button :loading="fetching" @click="onFetch">Fetch</n-button>
      <n-button :loading="diff.loading" @click="reloadDiff">重新比对</n-button>
      <n-button-group>
        <n-button
          :type="diff.viewMode === 'tree' ? 'primary' : 'default'"
          @click="diff.viewMode = 'tree'"
        >
          Tree
        </n-button>
        <n-button
          :type="diff.viewMode === 'list' ? 'primary' : 'default'"
          @click="diff.viewMode = 'list'"
        >
          List
        </n-button>
      </n-button-group>
    </n-space>

    <n-alert v-if="error" type="error" style="margin-top: 8px">{{ error }}</n-alert>
    <n-alert v-if="diff.error" type="error" style="margin-top: 8px">{{ diff.error }}</n-alert>

    <div class="diff-area" v-if="diff.branchDiff">
      <div class="stat-bar">
        共 {{ diff.branchDiff.files.length }} 个文件
        <span class="add">+{{ diff.branchDiff.totalAdditions }}</span>
        <span class="del">-{{ diff.branchDiff.totalDeletions }}</span>
        <span v-if="diff.branchDiff.truncated" class="warn">（结果已截断）</span>
      </div>
      <div class="panels">
        <div class="left">
          <FileChangeTree
            v-if="diff.viewMode === 'tree'"
            :files="diff.branchDiff.files"
            :selected="selectedPath"
            @select="onSelectFile"
          />
          <FileChangeList
            v-else
            :files="diff.branchDiff.files"
            :selected="selectedPath"
            @select="onSelectFile"
          />
        </div>
        <div class="right">
          <DiffViewer
            :repo-id="repo.current?.id ?? null"
            :target-ref="repo.target"
            :path="selectedPath"
            :file-diff="diff.fileDiff"
          />
        </div>
      </div>
      <CommitGraph :commits="commits" />
    </div>
    <n-empty
      v-else-if="!diff.loading"
      style="margin-top: 40px"
      description="选择两个分支后会自动比对"
    />
  </div>
  <n-empty v-else description="请在左侧选择工作区" />
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import {
  NSpace,
  NButton,
  NButtonGroup,
  NAlert,
  NEmpty,
  useMessage,
} from 'naive-ui';
import BranchPicker from '../components/BranchPicker.vue';
import FileChangeTree from '../components/FileChangeTree.vue';
import FileChangeList from '../components/FileChangeList.vue';
import DiffViewer from '../components/DiffViewer.vue';
import CommitGraph from '../components/CommitGraph.vue';
import type { Commit } from '../types';
import { useRepoStore } from '../stores/repo';
import { useWorkspacesStore } from '../stores/workspaces';
import { useDiffStore } from '../stores/diff';
import { gitApi } from '../api/git';

const route = useRoute();
const repo = useRepoStore();
const wsStore = useWorkspacesStore();
const diff = useDiffStore();
const message = useMessage();

const loading = ref(false);
const fetching = ref(false);
const error = ref<string | null>(null);
const selectedPath = ref<string | null>(null);
const commits = ref<Commit[]>([]);

async function loadFor(id: string) {
  error.value = null;
  diff.branchDiff = null;
  diff.fileDiff = null;
  selectedPath.value = null;
  commits.value = [];
  if (!wsStore.list.length) await wsStore.reload();
  repo.current = wsStore.list.find((w) => w.id === id) ?? null;
  repo.branches = [];
  repo.base = null;
  repo.target = null;
  if (!repo.current) return;
  loading.value = true;
  try {
    repo.branches = await gitApi.listBranches(id);
    const cur = await gitApi.currentBranch(id);
    repo.base = cur;
    repo.target =
      repo.branches.find((b) => b.kind === 'remote' && b.name.endsWith('/' + cur))?.name ?? null;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

async function reloadDiff() {
  if (!repo.current || !repo.base || !repo.target) {
    message.warning('请选择两个分支');
    return;
  }
  await diff.loadBranchDiff(repo.current.id, repo.base, repo.target);
  await loadCommits();
}

async function loadCommits() {
  if (!repo.current || !repo.base || !repo.target) {
    commits.value = [];
    return;
  }
  try {
    commits.value = await gitApi.logBetween(repo.current.id, repo.base, repo.target, 200);
  } catch {
    commits.value = [];
  }
}

async function onFetch() {
  if (!repo.current) return;
  fetching.value = true;
  try {
    await gitApi.fetch(repo.current.id);
    message.success('fetch 完成');
    await loadFor(repo.current.id);
  } catch (e: unknown) {
    const m = e instanceof Error ? e.message : String(e);
    message.error(m);
  } finally {
    fetching.value = false;
  }
}

async function onSelectFile(p: string) {
  selectedPath.value = p;
  if (repo.current && repo.base && repo.target) {
    await diff.loadFileDiff(repo.current.id, repo.base, repo.target, p);
  }
}

watch(() => route.params.id as string, (id) => id && loadFor(id), { immediate: true });
watch(
  [() => repo.base, () => repo.target],
  ([b, t]) => {
    if (b && t) reloadDiff();
  }
);

onMounted(() => {
  const id = route.params.id as string;
  if (id) loadFor(id);
});
</script>

<style scoped>
.repo-detail {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 8px;
}
.diff-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  border: 1px solid #2a2a2a;
  border-radius: 4px;
  overflow: hidden;
}
.stat-bar {
  padding: 6px 10px;
  border-bottom: 1px solid #2a2a2a;
  font-size: 12px;
}
.stat-bar .add { color: #6acf7c; margin-left: 8px; }
.stat-bar .del { color: #ff6a6a; margin-left: 6px; }
.stat-bar .warn { color: #e8c068; margin-left: 8px; }
.panels {
  flex: 1;
  display: grid;
  grid-template-columns: 40% 60%;
  gap: 8px;
  overflow: hidden;
  padding: 8px;
}
.left,
.right {
  border: 1px solid #2a2a2a;
  border-radius: 4px;
  overflow: auto;
}
</style>
