import { load, type Store } from "@tauri-apps/plugin-store";
import { isTauri } from "./ipc";

const STORE_FILE = "settings.json";

let storePromise: Promise<Store> | null = null;

async function getStore(): Promise<Store | null> {
  if (!isTauri()) {
    return null;
  }
  storePromise ??= load(STORE_FILE, { autoSave: true, defaults: {} });
  return storePromise;
}

export async function readSetting(key: string): Promise<string | null> {
  const store = await getStore();
  if (store) {
    const value = await store.get<string>(key);
    return value ?? null;
  }
  return localStorage.getItem(key);
}

export async function writeSetting(key: string, value: string): Promise<void> {
  const store = await getStore();
  if (store) {
    await store.set(key, value);
    return;
  }
  localStorage.setItem(key, value);
}
