import { getStore, Store } from "@tauri-apps/plugin-store";

export let storage: Store = null as unknown as Store;

let storageInterval = setInterval(async () => {
    if (!storage) {
        getStore("store.json").then((s) => {
            console.log("Storage initialized", s);
            storage = s!;
            if (storage) clearInterval(storageInterval);
        });
    } else {
        console.log("Storage already initialized");
        clearInterval(storageInterval);
    }
}, 250);