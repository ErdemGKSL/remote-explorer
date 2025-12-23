import { invoke } from "@tauri-apps/api/core";

export interface Terminal {
	id: string;
	name: string;
	path: string;
	history: TerminalLine[];
	currentCommand: string;
}

export interface TerminalLine {
	type: "command" | "output" | "error";
	content: string;
	timestamp: number;
}

class TerminalManager {
	terminals = $state<Map<string, Terminal>>(new Map());
	projectKey = $state<string>("");

	setProjectKey(key: string) {
		this.projectKey = key;
	}

	getTerminals() {
		return Array.from(this.terminals.values());
	}

	getTerminalsForPath(path: string) {
		return Array.from(this.terminals.values()).filter((t) => t.path === path);
	}

	getTerminal(id: string) {
		return this.terminals.get(id);
	}

	async createTerminal(name: string, path: string): Promise<string> {
		try {
			const id = await invoke<string>("create_terminal", {
				key: this.projectKey,
				name,
				path,
			});

			const terminal: Terminal = {
				id,
				name,
				path,
				history: [
					{
						type: "output",
						content: `Terminal "${name}" started at ${path}`,
						timestamp: Date.now(),
					},
				],
				currentCommand: "",
			};

			this.terminals.set(id, terminal);
			return id;
		} catch (error) {
			throw new Error(`Failed to create terminal: ${error}`);
		}
	}

	async executeCommand(terminalId: string, command: string): Promise<void> {
		const terminal = this.terminals.get(terminalId);
		if (!terminal) throw new Error("Terminal not found");

		// Add command to history
		terminal.history.push({
			type: "command",
			content: command,
			timestamp: Date.now(),
		});

		try {
			const [stdout, stderr, exitStatus] = await invoke<[string, string, number]>(
				"execute_terminal_command",
				{
					key: this.projectKey,
					terminalId,
					command,
				}
			);

			// Add output to history
			if (stdout.trim()) {
				terminal.history.push({
					type: "output",
					content: stdout,
					timestamp: Date.now(),
				});
			}

			if (stderr.trim()) {
				terminal.history.push({
					type: "error",
					content: stderr,
					timestamp: Date.now(),
				});
			}

			if (exitStatus !== 0 && !stderr.trim()) {
				terminal.history.push({
					type: "error",
					content: `Command exited with status ${exitStatus}`,
					timestamp: Date.now(),
				});
			}

			// Update terminal reference to trigger reactivity
			this.terminals.set(terminalId, { ...terminal });
		} catch (error) {
			terminal.history.push({
				type: "error",
				content: `Error: ${error}`,
				timestamp: Date.now(),
			});
			this.terminals.set(terminalId, { ...terminal });
		}
	}

	async closeTerminal(terminalId: string): Promise<void> {
		try {
			await invoke("close_terminal", {
				key: this.projectKey,
				terminalId,
			});
			this.terminals.delete(terminalId);
		} catch (error) {
			throw new Error(`Failed to close terminal: ${error}`);
		}
	}

	async loadTerminals(): Promise<void> {
		try {
			const terminals = await invoke<[string, string, string][]>("list_terminals", {
				key: this.projectKey,
			});

			for (const [id, name, path] of terminals) {
				if (!this.terminals.has(id)) {
					this.terminals.set(id, {
						id,
						name,
						path,
						history: [
							{
								type: "output",
								content: `Terminal "${name}" reconnected`,
								timestamp: Date.now(),
							},
						],
						currentCommand: "",
					});
				}
			}
		} catch (error) {
			console.error("Failed to load terminals:", error);
		}
	}

	clearAll() {
		this.terminals.clear();
	}
}

export const terminalManager = new TerminalManager();