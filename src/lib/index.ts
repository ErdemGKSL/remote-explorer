import { getStore, load, Store } from "@tauri-apps/plugin-store";
import { toast } from "svelte-sonner";

export let storage: Store = null as unknown as Store;

declare global {
    interface Window {
        storageInterval: number;
    }
}

if (window.storageInterval) {
    clearInterval(window.storageInterval);
}

window.storageInterval = setInterval(async () => {
    if (!storage) {
        getStore("store.json").then((s) => {
            console.log("Storage initialized", s);
            toast.success("Storage initialized");
            storage = s!;
            if (storage) clearInterval(window.storageInterval);
        }).catch((e) => {
            load("store.json");
            console.error("Failed to initialize storage", e);
        });
    } else {
        console.log("Storage already initialized");
        toast("Storage already initialized");
        clearInterval(window.storageInterval);
    }
}, 1000);