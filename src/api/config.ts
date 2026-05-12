import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, Workspace } from '../types';

export const configApi = {
  load: () => invoke<AppConfig>('config_load'),
  listWorkspaces: () => invoke<Workspace[]>('workspace_list'),
  createWorkspace: (args: { name: string; path: string; remoteName?: string }) =>
    invoke<Workspace>('workspace_create', { args }),
  updateWorkspace: (args: { id: string; patch: Partial<Workspace> }) =>
    invoke<Workspace>('workspace_update', { args }),
  deleteWorkspace: (id: string) => invoke<void>('workspace_delete', { args: { id } }),
  pickDir: () => invoke<string | null>('workspace_pick_dir'),
};
