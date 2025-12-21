<script lang="ts">
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogTrigger } from "$lib/components/ui/dialog";
	import { Drawer, DrawerContent, DrawerHeader, DrawerTitle, DrawerFooter, DrawerTrigger, DrawerClose } from "$lib/components/ui/drawer";
	import type { Snippet } from 'svelte';
	import { onMount } from "svelte";

	interface Props {
		open?: boolean;
		onOpenChange?: (open: boolean) => void;
		title: string;
		trigger: Snippet;
		children: Snippet;
		footer?: Snippet;
	}

	let { open = $bindable(false), onOpenChange, title, trigger, children, footer }: Props = $props();
	let isDesktop = $state(true);

	onMount(() => {
		// Check initial screen size
		isDesktop = window.matchMedia("(min-width: 768px)").matches;

		// Listen for screen size changes
		const mediaQuery = window.matchMedia("(min-width: 768px)");
		const handler = (e: MediaQueryListEvent) => {
			isDesktop = e.matches;
		};

		mediaQuery.addEventListener("change", handler);
		return () => mediaQuery.removeEventListener("change", handler);
	});

	function handleOpenChange(newOpen: boolean) {
		open = newOpen;
		onOpenChange?.(newOpen);
	}
</script>

{#if isDesktop}
	<Dialog bind:open onOpenChange={handleOpenChange}>
		<DialogTrigger>
			{@render trigger()}
		</DialogTrigger>
		<DialogContent class="sm:max-w-md">
			<DialogHeader>
				<DialogTitle>{title}</DialogTitle>
			</DialogHeader>
			{@render children()}
			<DialogFooter>
				{#if footer}
					{@render footer()}
				{/if}
			</DialogFooter>
		</DialogContent>
	</Dialog>
{:else}
	<Drawer bind:open onOpenChange={handleOpenChange}>
		<DrawerTrigger>
			{@render trigger()}
		</DrawerTrigger>
		<DrawerContent>
			<div class="mx-auto w-full max-w-sm flex flex-col max-h-[80vh]">
				<DrawerHeader>
					<DrawerTitle>{title}</DrawerTitle>
				</DrawerHeader>
				<div class="px-4 flex-1 overflow-y-auto">
					{@render children()}
				</div>
				<DrawerFooter>
					{#if footer}
						{@render footer()}
					{/if}
					<DrawerClose />
				</DrawerFooter>
			</div>
		</DrawerContent>
	</Drawer>
{/if}
