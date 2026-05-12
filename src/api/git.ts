import { invoke } from '@tauri-apps/api/core';
import type { Branch, BranchDiff, Commit, FileDiff } from '../types';

export const gitApi = {
  // P3 后端实现前，做一个容错壳，避免前端因命令未注册而崩
  validateRepo: async (path: string) => {
    try {
      return await invoke<{ valid: boolean; root?: string }>('git_validate_repo', { args: { path } });
    } catch {
      return { valid: true } as { valid: boolean; root?: string };
    }
  },
  listBranches: (repoId: string) => invoke<Branch[]>('git_list_branches', { args: { repoId } }),
  fetch: (repoId: string, remote?: string) =>
    invoke<void>('git_fetch', { args: { repoId, remote } }),
  fetchAndPull: (repoId: string) =>
    invoke<void>('git_fetch_and_pull', { args: { repoId } }),
  pullBranch: (repoId: string, branch: string) =>
    invoke<void>('git_pull_branch', { args: { repoId, branch } }),
  currentBranch: (repoId: string) => invoke<string>('git_current_branch', { args: { repoId } }),
  diffBranches: (repoId: string, base: string, target: string) =>
    invoke<BranchDiff>('git_diff_branches', { args: { repoId, base, target } }),
  fileDiff: (repoId: string, base: string, target: string, path: string) =>
    invoke<FileDiff>('git_file_diff', { args: { repoId, base, target, path } }),
  fileContent: (repoId: string, ref: string, path: string) =>
    invoke<string>('git_file_content', { args: { repoId, ref, path } }),
  logBetween: (repoId: string, base: string, target: string, limit?: number) =>
    invoke<Commit[]>('git_log_between', { args: { repoId, base, target, limit } }),
};
