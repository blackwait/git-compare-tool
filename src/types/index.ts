export interface Workspace {
  id: string;
  name: string;
  path: string;
  remoteName?: string;
  createdAt: number;
  updatedAt: number;
}

export interface Settings {
  theme: 'dark' | 'light' | 'auto';
  defaultView: 'tree' | 'list';
}

export interface AppConfig {
  version: number;
  workspaces: Workspace[];
  settings: Settings;
}

export interface Branch {
  name: string;
  kind: 'local' | 'remote';
  isHead: boolean;
  upstream?: string;
}

export type ChangeKind = 'added' | 'modified' | 'deleted' | 'renamed';

export interface FileChange {
  path: string;
  oldPath?: string;
  kind: ChangeKind;
  additions: number;
  deletions: number;
}

export interface BranchDiff {
  baseRef: string;
  targetRef: string;
  files: FileChange[];
  totalAdditions: number;
  totalDeletions: number;
  truncated?: boolean;
}

export interface DiffLine {
  kind: 'context' | 'add' | 'del';
  content: string;
  oldLineNo?: number;
  newLineNo?: number;
}

export interface DiffHunk {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
}

export interface FileDiff {
  path: string;
  oldPath?: string;
  isBinary?: boolean;
  hunks: DiffHunk[];
  truncated?: boolean;
}

export interface Commit {
  hash: string;
  shortHash: string;
  parents: string[];
  author: string;
  date: number;
  message: string;
}
