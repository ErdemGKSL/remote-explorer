<script lang="ts">
	import { onMount, tick } from "svelte";
	import { terminalManager, type Terminal } from "./terminal-store.svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { ScrollArea } from "$lib/components/ui/scroll-area";
	import * as Card from "$lib/components/ui/card";
	import { X, Terminal as TerminalIcon } from "@lucide/svelte";

	interface Props {
		terminal: Terminal;
	}

	let { terminal }: Props = $props();
	let commandInput = $state("");
	let scrollContainer: HTMLDivElement = $state(null!);
	let inputElement: HTMLInputElement = $state(null!);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!commandInput.trim()) return;

		const cmd = commandInput;
		commandInput = "";

		await terminalManager.executeCommand(terminal.id, cmd);
		await tick();
		scrollToBottom();
	}

	function scrollToBottom() {
		if (scrollContainer) {
			const scrollArea = scrollContainer.querySelector("[data-radix-scroll-area-viewport]");
			if (scrollArea) {
				scrollArea.scrollTop = scrollArea.scrollHeight;
			}
		}
	}

	async function handleClose() {
		await terminalManager.closeTerminal(terminal.id);
	}

	onMount(() => {
		scrollToBottom();
	});

	$effect(() => {
		// Watch for history changes
		terminal.history;
		tick().then(() => scrollToBottom());
	});
</script>

<Card.Root class="flex flex-col h-full">
	<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2 px-4 py-2 border-b">
		<div class="flex items-center gap-2">
			<TerminalIcon class="h-4 w-4 text-muted-foreground" />
			<Card.Title class="text-sm font-medium">{terminal.name}</Card.Title>
			<span class="text-xs text-muted-foreground">({terminal.path})</span>
		</div>
		<Button variant="ghost" size="icon" class="h-6 w-6" onclick={handleClose}>
			<X class="h-4 w-4" />
		</Button>
	</Card.Header>
	<Card.Content class="flex-1 p-0 flex flex-col min-h-0">
		<ScrollArea class="flex-1 px-4" bind:ref={scrollContainer}>
			<div class="py-2 space-y-1 font-mono text-sm">
				{#each terminal.history as line}
					{#if line.type === "command"}
						<div class="flex gap-2 text-green-600 dark:text-green-400">
							<span class="select-none">$</span>
							<span>{line.content}</span>
						</div>
					{:else if line.type === "error"}
						<div class="text-red-600 dark:text-red-400 whitespace-pre-wrap">
							{line.content}
						</div>
					{:else}
						<div class="text-foreground whitespace-pre-wrap">
							{line.content}
						</div>
					{/if}
				{/each}
			</div>
		</ScrollArea>
		<div class="border-t px-4 py-2">
			<form onsubmit={handleSubmit} class="flex gap-2">
				<span class="text-green-600 dark:text-green-400 font-mono text-sm flex items-center select-none">$</span>
				<Input
					bind:value={commandInput}
					bind:ref={inputElement}
					placeholder="Enter command..."
					class="flex-1 font-mono text-sm"
					autocomplete="off"
				/>
			</form>
		</div>
	</Card.Content>
</Card.Root>