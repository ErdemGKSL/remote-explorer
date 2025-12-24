<script lang="ts">
	import AnsiToHtml from 'ansi-to-html';
	import { onMount, tick } from "svelte";
	import { terminalManager, type Terminal } from "./terminal-store.svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { ScrollArea } from "$lib/components/ui/scroll-area";
	import { X, Terminal as TerminalIcon, Trash2 } from "@lucide/svelte";

	const ansi = new AnsiToHtml();
	
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

	function cleanTerminalOutput(text: string): string {
		return text
			.replace("[?2004h00ac05c254fc:~$", "")
			.replace("00ac05c254fc:~$ ", "") // Remove prompt if exists
			.replace(/\x1b\[\?2004h/g, '')  // Remove enable bracketed paste
			.replace(/\x1b\[\?2004l/g, '')  // Remove disable bracketed paste
			.replace(/\x1b\[\?1h/g, '')     // Remove other cursor modes
			.replace(/\x1b\[\?1l/g, '')
			.replace(/\x1b\[H/g, '')        // Remove cursor home
			.replace(/\x1b\[2J/g, '')       // Remove clear screen
			.replace(/\x1b\[K/g, '');       // Remove clear line
	}

	async function handleClose() {
		await terminalManager.closeTerminal(terminal.id);
	}

	async function handleClear() {
		terminal.history = [];
	}

	onMount(() => {
		scrollToBottom();
		// Auto-focus input
		setTimeout(() => inputElement?.focus(), 100);
	});

	$effect(() => {
		// Watch for history changes
		terminal.history;
		tick().then(() => scrollToBottom());
	});
</script>

<div class="flex flex-col h-full bg-background ">
	<!-- Terminal Header -->
	<div class="flex items-center justify-between px-4 py-2 border-b bg-muted/30">
		<div class="flex items-center gap-2 flex-1 min-w-0">
			<TerminalIcon class="h-4 w-4 text-muted-foreground shrink-0" />
			<span class="text-sm font-medium truncate">{terminal.id}</span>
			<span class="text-xs text-muted-foreground truncate">· {terminal.path}</span>
		</div>
		<div class="flex items-center gap-1">
			<Button 
				variant="ghost" 
				size="icon" 
				class="h-7 w-7" 
				onclick={handleClear}
				title="Clear terminal"
			>
				<Trash2 class="h-3.5 w-3.5" />
			</Button>
			<Button 
				variant="ghost" 
				size="icon" 
				class="h-7 w-7" 
				onclick={handleClose}
				title="Close terminal"
			>
				<X class="h-3.5 w-3.5" />
			</Button>
		</div>
	</div>

	<!-- Terminal Content -->
	<div class="flex-1 flex flex-col min-h-0 h-full">
		<ScrollArea class="flex-1 px-4 h-[calc(100%-3.5rem)]" bind:ref={scrollContainer}>
			<div class="py-2 space-y-0.5 font-roboto-mono! font-mono text-sm">
				{#each terminal.history as line, i (i)}
					{#if line.type === "command"}
						<div class="flex gap-2 text-green-600 dark:text-green-400">
							<span class="select-none">$</span>
							<span class="flex-1">{line.content}</span>
						</div>
					{:else if line.type === "error"}
						{@const c = ansi.toHtml(cleanTerminalOutput(line.content)).trim()}
						{#if c.length > 0}
							<div class="text-red-600 dark:text-red-400 whitespace-pre-wrap">
								{@html c}
							</div>
						{/if}
					{:else}
						{@const c = ansi.toHtml(cleanTerminalOutput(line.content)).trim()}
						{#if c.length > 0}
							<div class="text-foreground whitespace-pre-wrap leading-relaxed">
								{@html c}
							</div>
						{/if}
					{/if}
				{/each}
				{#if terminal.history.length === 0}
					<div class="text-muted-foreground py-4">
						Terminal ready. Type a command and press Enter.
					</div>
				{/if}
			</div>
		</ScrollArea>

		<!-- Terminal Input -->
		<div class="border-t px-4 py-2 bg-muted/10 h-14">
			<form onsubmit={handleSubmit} class="flex gap-2 items-center">
				<span class="text-green-600 dark:text-green-400 font-mono text-sm select-none">$</span>
				<Input
					bind:value={commandInput}
					bind:ref={inputElement}
					placeholder="Enter command..."
					class="flex-1 font-mono text-sm h-8 border-0 focus-visible:ring-0 bg-transparent px-0"
					autocomplete="off"
					spellcheck={false}
				/>
			</form>
		</div>
	</div>
</div>