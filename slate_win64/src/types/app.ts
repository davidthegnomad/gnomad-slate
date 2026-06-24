/** Shared app-level types for Gnomad Slate (Windows port). */

export interface Note {
  filename: string;
  title: string;
  content: string;
  is_ephemeral: boolean;
  last_modified: number;
}

export interface ClipboardEntry {
  text: string;
  timestamp: number;
}

export interface SecretMeta {
  index: number;
  name: string;
  path: string;
}

export interface AppVersionInfo {
  version: string;
  channel: string;
  display_version: string;
}

export type EditorTab = "all" | "scratch" | "vault" | "secrets";
export type EditorViewMode = "edit" | "preview" | "split";
