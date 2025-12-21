import { getStore, Store } from "@tauri-apps/plugin-store";

export let storage: Store = null as unknown as Store;

(async () => {
    getStore("store.json").then((s) => {
        storage = s!;
    });
});