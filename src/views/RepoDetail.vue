<template>
  <div v-if="repo.current" class="repo-detail">
    <n-space align="center" :size="12" style="flex-wrap: wrap">
      <h2 style="margin: 0">{{ repo.current.name }}</h2>
      <BranchPicker :branches="repo.branches" v-model="repo.base" :loading="loading" @copy-name="onCopyBranchName" @update-branch="onUpdateBranch" />
      <n-button text style="font-size: 18px; cursor: pointer" @click="onSwapBranches">⇄</n-button>
      <BranchPicker :branches="repo.branches" v-model="repo.target" :loading="loading" @copy-name="onCopyBranchName" @update-branch="onUpdateBranch" />
      <n-button :loading="fetching" @click="onFetch">Fetch</n-button>
      <n-button :loading="pulling" type="primary" @click="onPull">一键更新</n-button>
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
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useRoute, onBeforeRouteUpdate } from 'vue-router';
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
import { useRepoStore, getSavedBranches } from '../stores/repo';
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
const pulling = ref(false);
const swapping = ref(false);
const error = ref<string | null>(null);
const selectedPath = ref<string | null>(null);
const commits = ref<Commit[]>([]);

async function loadFor(id: string) {
  repo.pauseSave();
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
  if (!repo.current) { repo.resumeSave(); return; }
  loading.value = true;
  try {
    repo.branches = await gitApi.listBranches(id);
    // 优先恢复上次保存的分支选择
    const saved = getSavedBranches(id);
    if (saved.base && repo.branches.some((b) => b.name === saved.base)) {
      repo.base = saved.base;
    } else {
      repo.base = await gitApi.currentBranch(id);
    }
    if (saved.target && repo.branches.some((b) => b.name === saved.target)) {
      repo.target = saved.target;
    } else {
      const cur = repo.base;
      repo.target =
        repo.branches.find((b) => b.kind === 'remote' && b.name.endsWith('/' + cur))?.name ?? null;
    }
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
    repo.resumeSave();
    // 切换仓库后主动触发比对
    if (repo.base && repo.target) {
      reloadDiff();
    }
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
  const prevBase = repo.base;
  const prevTarget = repo.target;
  try {
    await gitApi.fetch(repo.current.id);
    message.success('fetch 完成');
    repo.branches = await gitApi.listBranches(repo.current.id);
    repo.base = repo.branches.some((b) => b.name === prevBase) ? prevBase : null;
    repo.target = repo.branches.some((b) => b.name === prevTarget) ? prevTarget : null;
  } catch (e: unknown) {
    const m = e instanceof Error ? e.message : String(e);
    message.error(m);
  } finally {
    fetching.value = false;
  }
}

async function onPull() {
  if (!repo.current) return;
  pulling.value = true;
  const prevBase = repo.base;
  const prevTarget = repo.target;
  try {
    await gitApi.fetchAndPull(repo.current.id);
    message.success('更新完成');
    repo.branches = await gitApi.listBranches(repo.current.id);
    repo.base = repo.branches.some((b) => b.name === prevBase) ? prevBase : null;
    repo.target = repo.branches.some((b) => b.name === prevTarget) ? prevTarget : null;
    if (repo.base && repo.target) {
      await reloadDiff();
    }
  } catch (e: unknown) {
    const m = e instanceof Error ? e.message : String(e);
    message.error(m);
  } finally {
    pulling.value = false;
  }
}

async function onSelectFile(p: string) {
  selectedPath.value = p;
  if (repo.current && repo.base && repo.target) {
    await diff.loadFileDiff(repo.current.id, repo.base, repo.target, p);
  }
}

function onCopyBranchName(branchName: string) {
  navigator.clipboard.writeText(branchName);
  message.success(`已复制: ${branchName}`);
}

function onSwapBranches() {
  if (!repo.base && !repo.target) return;
  swapping.value = true;
  const tmp = repo.base;
  repo.base = repo.target;
  repo.target = tmp;
  if (repo.base && repo.target) {
    reloadDiff();
  }
  setTimeout(() => { swapping.value = false; }, 0);
}

async function onUpdateBranch(branchName: string) {
  if (!repo.current) return;
  try {
    await gitApi.pullBranch(repo.current.id, branchName);
    message.success(`分支 ${branchName} 更新完成`);
  } catch (e: unknown) {
    const m = e instanceof Error ? e.message : String(e);
    message.error(`更新失败: ${m}`);
  }
}

/** 键盘上下键切换文件 */
function onKeyDown(e: KeyboardEvent) {
  if (!diff.branchDiff || !diff.branchDiff.files.length) return;
  if (e.key !== 'ArrowUp' && e.key !== 'ArrowDown') return;
  // 避免在输入框中触发
  const tag = (e.target as HTMLElement)?.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA') return;
  e.preventDefault();
  const files = diff.branchDiff.files;
  const curIdx = selectedPath.value ? files.findIndex((f) => f.path === selectedPath.value) : -1;
  let nextIdx: number;
  if (e.key === 'ArrowDown') {
    nextIdx = curIdx < files.length - 1 ? curIdx + 1 : 0;
  } else {
    nextIdx = curIdx > 0 ? curIdx - 1 : files.length - 1;
  }
  onSelectFile(files[nextIdx].path);
}

onMounted(() => document.addEventListener('keydown', onKeyDown));
onUnmounted(() => document.removeEventListener('keydown', onKeyDown));

watch(() => route.params.id as string, (id) => id && loadFor(id), { immediate: true });

onBeforeRouteUpdate((to) => {
  const id = to.params.id as string;
  if (id) loadFor(id);
});

watch(
  [() => repo.base, () => repo.target],
  ([b, t], [oldB, oldT]) => {
    if (!loading.value && !swapping.value && b && t && (b !== oldB || t !== oldT)) {
      reloadDiff();
    }
  }
);
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
  padding: 8px 12px;
  border-bottom: 1px solid #2a2a2a;
  font-size: 12px;
  color: #ffffff;
  background: #1a1a1a;
}
.stat-bar .add { color: #56d364; margin-left: 8px; font-weight: 600; }
.stat-bar .del { color: #ff7b72; margin-left: 6px; font-weight: 600; }
.stat-bar .warn { color: #ffbf47; margin-left: 8px; }
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
  background: #1a1a1a;
}
</style>
