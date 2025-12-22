<script lang="ts">
	import { platform } from "@tauri-apps/plugin-os"
	import { getCurrentWindow } from "@tauri-apps/api/window";
	import { Button } from "$lib/components/ui/button";
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
    import { Toaster } from '$lib/components/ui/sonner';
	import './layout.css';
    import { Minus, Square, X } from "@lucide/svelte";

	let currentPlatform = platform();
	let isMobile = currentPlatform === 'android' || currentPlatform === 'ios';
	const appWindow = getCurrentWindow();
	let showButtons = $state(true);

	const { children } = $props();

	async function minimize() {
		await appWindow.minimize();
	}

	async function maximize() {
		await appWindow.maximize();
	}

	async function closeWindow() {
		await appWindow.close();
	}

	onMount(async () => {
		if (isMobile) {
			showButtons = false;
		} else if (currentPlatform === 'linux') {
			try {
				const [xdgDesktop, desktopSession] = await invoke<[string | null, string | null]>('get_desktop_environment');
				console.log('XDG_CURRENT_DESKTOP:', xdgDesktop);
				console.log('DESKTOP_SESSION:', desktopSession);
				const isHyprland = 
					(xdgDesktop && xdgDesktop.toLowerCase().includes('hyprland')) ||
					(desktopSession && desktopSession.toLowerCase().includes('hyprland'));
				
				if (isHyprland) {
					showButtons = false;
				}
			} catch {
				// Ignore errors
			}
		}
	});
</script>

<Toaster />
<div class="flex h-screen flex-col pt-8 md:pt-0 pb-2 md:pb-0">
	{#if showButtons}
		<div 
			class="flex items-center justify-between bg-background border-b px-4 py-2 select-none h-12"
			style="-webkit-app-region: drag;"
		>
			<span class="text-sm font-medium">Remote Explorer</span>
			<div class="flex items-center gap-1">
				<Button variant="ghost" size="icon" onclick={minimize} style="-webkit-app-region: no-drag;">
					<span class="text-xs"><Minus /></span>
				</Button>
				<Button variant="ghost" size="icon" onclick={maximize} style="-webkit-app-region: no-drag;">
					<span class="text-xs"><Square /></span>
				</Button>
				<Button variant="ghost" size="icon" onclick={closeWindow} style="-webkit-app-region: no-drag;">
					<span class="text-xs"><X /></span>
				</Button>
			</div>
		</div>
	{/if}
	
	<div class="flex-1 flex flex-col overflow-hidden h-[calc(100%-3rem)]">
		{@render children()}
	</div>
</div>

