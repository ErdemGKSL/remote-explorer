<script lang="ts">
	import {} from "@tauri-apps/plugin-clipboard-manager";
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { page } from "$app/state";
	import {
		Folder,
		File,
		ChevronLeft,
		House,
		RefreshCw,
		Unplug,
		Terminal as TerminalIcon,
		Plus,
	} from "@lucide/svelte";
	import { Button } from "$lib/components/ui/button";
	import { Separator } from "$lib/components/ui/separator";
	import { ScrollArea } from "$lib/components/ui/scroll-area";
	import * as ContextMenu from "$lib/components/ui/context-menu";
	import * as Breadcrumb from "$lib/components/ui/breadcrumb";
	import { loadConnection } from "./connection.svelte";
	import Input from "$lib/components/ui/input/input.svelte";
	import ResponsiveDialog from "$lib/components/ui/responsive-dialog/responsive-dialog.svelte";
	import { getCurrentWindow } from "@tauri-apps/api/window";
	import { platform } from "@tauri-apps/plugin-os";
	import Terminal from "./terminal.svelte";
	import { terminalManager } from "./terminal-store.svelte";

	interface DirEntry {
		name: string;
		is_dir: boolean;
		size: string;
		permissions: string;
		modified: string;
	}

	let currentPath = $state("");
	let entries = $state<DirEntry[]>([]);
	let loading = $state(false);
	let error = $state("");
	let projectKey = $derived(page.url.searchParams.get("key") || "");
	let pathInputFocused = $state(false);
	let pathInputValue = $state("");
	let creatingType = $state<"file" | "folder" | null>(null);
	let newItemName = $state("");
	let showDeleteDialog = $state(false);
	let selectedEntry = $state<DirEntry | null>(null);
	let isMobile = $state(false);
	let showNewTerminalDialog = $state(false);
	let newTerminalName = $state("");
	const appWindow = getCurrentWindow();

	// Get terminals for current path
	let currentPathTerminals = $derived(
		terminalManager.getTerminalsForPath(currentPath)
	);

	async function loadDirectory(path: string) {
		loading = entries.length === 0;
		error = "";

		try {
			const result = await invoke<DirEntry[]>("get_dir_contents", {
				key: projectKey,
				path,
			});
			console.log("Directory contents for", path, result);
			entries = result;
			currentPath = path;
			pathInputValue = path;
		} catch (e) {
			error = String(e);
			entries = [];
		} finally {
			loading = false;
		}
	}

	async function refresh() {
		if (currentPath) {
			await loadDirectory(currentPath);
		}
	}

	function handlePathKeydown(event: KeyboardEvent) {
		if (event.key === "Enter") {
			event.preventDefault();
			loadDirectory(pathInputValue);
			(event.target as HTMLInputElement).blur();
		} else if (event.key === "Escape") {
			pathInputValue = currentPath;
			(event.target as HTMLInputElement).blur();
		}
	}

	function navigateToBreadcrumb(index: number) {
		const parts = currentPath.split("/").filter((p) => p);
		if (index === -1) {
			loadDirectory("/");
		} else {
			const newPath = "/" + parts.slice(0, index + 1).join("/");
			loadDirectory(newPath);
		}
	}

	async function navigateToEntry(entry: DirEntry) {
		if (entry.is_dir) {
			const newPath =
				currentPath === "/"
					? `/${entry.name}`
					: `${currentPath}/${entry.name}`;
			await loadDirectory(newPath);
		}
	}

	async function navigateUp() {
		if (currentPath === "/") return;

		const parts = currentPath.split("/").filter((p) => p);
		parts.pop();
		const newPath = parts.length === 0 ? "/" : "/" + parts.join("/");
		await loadDirectory(newPath);
	}

	async function navigateToHome() {
		const pwd = await invoke<string>("get_current_pwd", {
			key: projectKey,
		});
		await loadDirectory(pwd);
	}

	async function createItem() {
		if (!newItemName.trim() || !creatingType) {
			cancelCreating();
			return;
		}

		try {
			const fullPath =
				currentPath === "/"
					? `/${newItemName}`
					: `${currentPath}/${newItemName}`;

			if (creatingType === "file") {
				await invoke("create_file", {
					key: projectKey,
					path: fullPath,
				});
			} else {
				await invoke("create_folder", {
					key: projectKey,
					path: fullPath,
				});
			}

			creatingType = null;
			newItemName = "";
			await refresh();
		} catch (e) {
			error = String(e);
			creatingType = null;
			newItemName = "";
		}
	}

	function startCreating(type: "file" | "folder") {
		creatingType = type;
		newItemName = "";
		setTimeout(() => {
			const input = document.querySelector(
				".creating-item-input",
			) as HTMLInputElement;
			input?.focus();
		}, 0);
	}

	function cancelCreating() {
		creatingType = null;
		newItemName = "";
	}

	function openDeleteDialog(entry: DirEntry) {
		selectedEntry = entry;
		showDeleteDialog = true;
	}

	async function confirmDelete() {
		let entry = selectedEntry;
		showDeleteDialog = false;
		await new Promise((r) => setTimeout(r, 150));
		selectedEntry = null;
		if (entry) {
			await deleteItem(entry);
		}
	}

	async function deleteItem(entry: DirEntry) {
		try {
			const fullPath =
				currentPath === "/"
					? `/${entry.name}`
					: `${currentPath}/${entry.name}`;

			await invoke("delete_item", {
				key: projectKey,
				path: fullPath,
				isDir: entry.is_dir,
			});

			await refresh();
		} catch (e) {
			error = String(e);
		}
	}

	async function closeConnection() {
		try {
			await invoke("close_project", { key: projectKey });
		} catch (e) {
			error = String(e);
		}
	}

	function openNewTerminalDialog() {
		newTerminalName = `Terminal ${currentPathTerminals.length + 1}`;
		showNewTerminalDialog = true;
	}

	async function createNewTerminal() {
		if (!newTerminalName.trim()) return;

		try {
			await terminalManager.createTerminal(newTerminalName, currentPath);
			showNewTerminalDialog = false;
			newTerminalName = "";
		} catch (e) {
			error = String(e);
		}
	}

	onMount(() => {
		isMobile = platform() === "android" || platform() === "ios";

		(async () => {
			try {
				// Set project key for terminal manager
				terminalManager.setProjectKey(projectKey);

				// Load connection info first
				await loadConnection(projectKey);

				// Load existing terminals
				await terminalManager.loadTerminals();

				// Then load directory
				const pwd = await invoke<string>("get_current_pwd", {
					key: projectKey,
				});
				await loadDirectory(pwd);
			} catch (e) {
				error = String(e);
			}
		})();

		return () => {
			// Cleanup terminals when leaving
			terminalManager.clearAll();
		};
	});
</script>

<svelte:document
	onpaste={() => {
		// handle file pasting in the future
	}}
/>

<div class="flex h-full flex-col">
	<!-- Toolbar -->
	<div class="flex items-center gap-2 border-b p-3 h-12">
		<Button
			variant="ghost"
			size="icon"
			onclick={refresh}
			disabled={loading}
		>
			<RefreshCw class="h-5 w-5" />
		</Button>
		<Button
			variant="ghost"
			size="icon"
			onclick={navigateUp}
			disabled={currentPath === "/" || loading}
		>
			<ChevronLeft class="h-5 w-5" />
		</Button>
		<Button
			variant="ghost"
			size="icon"
			onclick={navigateToHome}
			disabled={loading}
		>
			<House class="h-5 w-5" />
		</Button>
		<Separator orientation="vertical" class="h-6" />
		<ContextMenu.Root>
			<ContextMenu.Trigger class="flex-1">
				<div class="flex-1">
					{#if pathInputFocused}
						<Input
							type="text"
							bind:value={pathInputValue}
							onfocus={() => (pathInputFocused = true)}
							onblur={() => (pathInputFocused = false)}
							onkeydown={handlePathKeydown}
							disabled={loading}
						/>
					{:else}
						<Breadcrumb.Root>
							<Breadcrumb.List>
								{#if currentPath === "/"}
									<Breadcrumb.Item>
										<Breadcrumb.Page>/</Breadcrumb.Page>
									</Breadcrumb.Item>
								{:else}
									<Breadcrumb.Item
										class="text-muted-foreground hover:text-foreground transition-colors cursor-pointer select-none"
										onclick={(e) => {
											e.stopPropagation();
											navigateToBreadcrumb(-1);
										}}
									>
										/
									</Breadcrumb.Item>
									{@const parts = currentPath
										.split("/")
										.filter((p) => p)}
									{#each parts as part, i}
										<Breadcrumb.Separator />
										<Breadcrumb.Item
											class="text-muted-foreground hover:text-foreground px-1 transition-colors select-none {i === parts.length - 1 ? 'cursor-default' : 'cursor-pointer'}"
											onclick={(e) => {
												if (i === parts.length - 1)
													return;
												e.stopPropagation();
												navigateToBreadcrumb(i);
											}}
										>
											{#if i === parts.length - 1}
												<Breadcrumb.Page
													>{part}</Breadcrumb.Page
												>
											{:else}
												{part}
											{/if}
										</Breadcrumb.Item>
									{/each}
								{/if}
							</Breadcrumb.List>
						</Breadcrumb.Root>
					{/if}
				</div>
			</ContextMenu.Trigger>
			<ContextMenu.Content>
				<ContextMenu.Item
					onclick={() => {
						pathInputFocused = true;
						setTimeout(() => {
							const input = document.querySelector(
								'input[type="text"]',
							) as HTMLInputElement;
							input?.focus();
							input?.select();
						}, 0);
					}}
				>
					Edit Path
				</ContextMenu.Item>
			</ContextMenu.Content>
		</ContextMenu.Root>
		<Button variant="ghost" size="icon" onclick={openNewTerminalDialog}>
			<TerminalIcon class="h-5 w-5" />
		</Button>
		<Button variant="ghost" size="icon" onclick={closeConnection}>
			<Unplug class="h-5 w-5" />
		</Button>
	</div>

	<!-- Main Content Area -->
	<div class="flex-1 flex flex-col min-h-0">
		<!-- Terminals Section -->
		{#if currentPathTerminals.length > 0}
			<div class="border-b">
				<div class="p-2 space-y-2 max-h-96 overflow-y-auto">
					<div class="grid gap-2 {currentPathTerminals.length === 1 ? 'grid-cols-1' : 'grid-cols-2'}">
						{#each currentPathTerminals as terminal (terminal.id)}
							<div class="h-64">
								<Terminal {terminal} />
							</div>
						{/each}
					</div>
				</div>
			</div>
		{/if}

		<!-- File List -->
		{#if error}
			<div class="flex flex-1 items-center justify-center">
				<div class="text-center">
					<p class="text-sm text-destructive">{error}</p>
					<Button variant="outline" class="mt-4" onclick={navigateToHome}>
						Go to Home
					</Button>
				</div>
			</div>
		{:else if loading}
			<div class="flex flex-1 items-center justify-center">
				<div class="text-sm text-muted-foreground">Loading...</div>
			</div>
		{:else}
			<ScrollArea class="flex-1">
				<ContextMenu.Root>
					<ContextMenu.Trigger class="w-full h-full">
						<div class="divide-y min-h-full">
							{#each entries as entry (entry.name)}
								<ContextMenu.Root>
									<ContextMenu.Trigger>
										<button
											class="flex w-full items-center gap-3 px-4 py-2 hover:bg-accent transition-colors"
											onclick={() => navigateToEntry(entry)}
											ondblclick={() =>
												entry.is_dir &&
												navigateToEntry(entry)}
										>
											{#if entry.is_dir}
												<Folder
													class="h-5 w-5 text-blue-500"
												/>
											{:else}
												<File
													class="h-5 w-5 text-muted-foreground"
												/>
											{/if}
											<div class="flex-1 text-left">
												<div class="text-sm font-medium">
													{entry.name}
												</div>
												<div
													class="text-xs text-muted-foreground"
												>
													{entry.permissions} · {entry.size}
													· {entry.modified}
												</div>
											</div>
										</button>
									</ContextMenu.Trigger>
									<ContextMenu.Content>
										{#if entry.is_dir}
											<ContextMenu.Item
												onclick={() =>
													navigateToEntry(entry)}
												>Open</ContextMenu.Item
											>
											<ContextMenu.Separator />
										{/if}
										<ContextMenu.Item>Copy</ContextMenu.Item>
										<ContextMenu.Item>Cut</ContextMenu.Item>
										<ContextMenu.Item>Paste</ContextMenu.Item>
										<ContextMenu.Separator />
										<ContextMenu.Item>Rename</ContextMenu.Item>
										<ContextMenu.Item
											class="text-destructive"
											onclick={() => openDeleteDialog(entry)}
											>Delete</ContextMenu.Item
										>
									</ContextMenu.Content>
								</ContextMenu.Root>
							{/each}

							<!-- Inline creating entry -->
							{#if creatingType}
								<div
									class="flex w-full items-center gap-3 px-4 py-2 bg-accent/50"
								>
									{#if creatingType === "folder"}
										<Folder class="h-5 w-5 text-blue-500" />
									{:else}
										<File
											class="h-5 w-5 text-muted-foreground"
										/>
									{/if}
									<Input
										type="text"
										bind:value={newItemName}
										onkeydown={(e) => {
											if (e.key === "Enter") {
												e.preventDefault();
												createItem();
											} else if (e.key === "Escape") {
												e.preventDefault();
												cancelCreating();
											}
										}}
										onblur={cancelCreating}
										placeholder="Enter name..."
									/>
								</div>
							{/if}

							{#if entries.length === 0 && !creatingType}
								<div class="flex h-96 items-center justify-center">
									<p class="text-sm text-muted-foreground">
										Empty directory
									</p>
								</div>
							{/if}
						</div>
					</ContextMenu.Trigger>
					<ContextMenu.Content>
						<ContextMenu.Item onclick={() => startCreating("file")}>
							Create File
						</ContextMenu.Item>
						<ContextMenu.Item onclick={() => startCreating("folder")}>
							Create Folder
						</ContextMenu.Item>
						<ContextMenu.Separator />
						<ContextMenu.Item onclick={openNewTerminalDialog}>
							<TerminalIcon class="h-4 w-4 mr-2" />
							New Terminal Here
						</ContextMenu.Item>
					</ContextMenu.Content>
				</ContextMenu.Root>
			</ScrollArea>
		{/if}
	</div>

	<!-- Delete Confirmation Dialog -->
	<ResponsiveDialog
		bind:open={showDeleteDialog}
		title="Delete Item"
		onOpenChange={(open) => {
			if (!open) selectedEntry = null;
		}}
	>
		{#snippet trigger()}
			<!-- svelte-ignore element_invalid_self_closing_tag -->
			<span class="hidden" />
		{/snippet}
		{#snippet children()}
			<p>
				Are you sure you want to delete "{selectedEntry?.name}"? This
				action cannot be undone.
			</p>
		{/snippet}
		{#snippet footer()}
			<Button variant="outline" onclick={() => (showDeleteDialog = false)}
				>Cancel</Button
			>
			<Button variant="destructive" onclick={confirmDelete}>Delete</Button
			>
		{/snippet}
	</ResponsiveDialog>

	<!-- New Terminal Dialog -->
	<ResponsiveDialog
		bind:open={showNewTerminalDialog}
		title="New Terminal"
	>
		{#snippet trigger()}
			<!-- svelte-ignore element_invalid_self_closing_tag -->
			<span class="hidden" />
		{/snippet}
		{#snippet children()}
			<div class="space-y-4">
				<div>
					<label for="terminal-name" class="text-sm font-medium">
						Terminal Name
					</label>
					<Input
						id="terminal-name"
						bind:value={newTerminalName}
						placeholder="Enter terminal name..."
						onkeydown={(e) => {
							if (e.key === "Enter") {
								e.preventDefault();
								createNewTerminal();
							}
						}}
					/>
				</div>
				<div class="text-sm text-muted-foreground">
					Path: {currentPath}
				</div>
			</div>
		{/snippet}
		{#snippet footer()}
			<Button variant="outline" onclick={() => (showNewTerminalDialog = false)}
				>Cancel</Button
			>
			<Button onclick={createNewTerminal}>Create</Button>
		{/snippet}
	</ResponsiveDialog>
</div>