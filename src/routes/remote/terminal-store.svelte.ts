import { invoke } from "@tauri-apps/api/core";

export interface TerminalLine {
	type: "command" | "output" | "error";
	content: string;
}

export interface Terminal {
	id: string;
	path: string;
	history: TerminalLine[];
	rawContent: string; // Store raw terminal output
	lastProcessedLength: number; // Track what we've already processed
}

class TerminalManager {
	terminals = $state<Record<string, Terminal>>({});
	projectKey = $state<string>("");

	setProjectKey(key: string) {
		this.projectKey = key;
	}

	async createTerminal(path: string): Promise<Terminal> {
		try {
			console.log("Creating terminal at path:", path);
			const terminalId = await invoke<string>("create_terminal", {
				key: this.projectKey,
				path,
			});

			console.log("Terminal created with ID:", terminalId);

			const terminal: Terminal = {
				id: terminalId,
				path,
				history: [],
				rawContent: "",
				lastProcessedLength: 0,
			};

			// Add to map
			this.terminals[terminalId] = terminal;

			// Start polling for this terminal
			this.startPolling(terminalId);

			return terminal;
		} catch (error) {
			console.error("Failed to create terminal:", error);
			throw error;
		}
	}

	async loadTerminals() {
		try {
			const terminalList = await invoke<Array<[string, string]>>("list_terminals", {
				key: this.projectKey,
			});

			console.log("Loaded terminals:", terminalList);

			// Clear existing terminals
			this.terminals = {};

			// Add loaded terminals
			for (const [id, path] of terminalList) {
				const terminal: Terminal = {
					id,
					path,
					history: [],
					rawContent: "",
					lastProcessedLength: 0,
				};
				this.terminals[id] = terminal;
				
				// Load existing content
				try {
					const content = await invoke<string>("get_terminal_content", {
						key: this.projectKey,
						terminalId: id,
					});
					
					terminal.rawContent = content;
					terminal.history = this.parseTerminalOutput(content);
					terminal.lastProcessedLength = content.length;
				} catch (e) {
					console.error(`Failed to load content for terminal ${id}:`, e);
				}

				// Start polling
				this.startPolling(id);
			}

			// Force reactivity update
			this.terminals = { ...this.terminals };
		} catch (error) {
			console.error("Failed to load terminals:", error);
		}
	}

	async executeCommand(terminalId: string, command: string) {
		const terminal = this.terminals[terminalId];
		if (!terminal) {
			console.error("Terminal not found:", terminalId);
			return;
		}

		try {
			// Add command to history for display
			terminal.history.push({
				type: "command",
				content: command,
			});

			// Execute command
			await invoke("execute_terminal_command", {
				key: this.projectKey,
				terminalId,
				command,
			});

			console.log("Command executed:", command);

			// Poll for output immediately after a short delay
			setTimeout(() => this.pollTerminalOutput(terminalId), 200);
		} catch (error) {
			console.error("Failed to execute command:", error);
			terminal.history.push({
				type: "error",
				content: `Error: ${error}`,
			});
		}
	}

	private parseTerminalOutput(content: string): TerminalLine[] {
		if (!content) return [];
		
		// Split by lines but keep the terminal output continuous
		const lines: TerminalLine[] = [];
		const contentLines = content.split('\n');
		
		for (const line of contentLines) {
			if (line.trim()) {
				lines.push({
					type: "output",
					content: line,
				});
			}
		}
		
		return lines;
	}

	private startPolling(terminalId: string) {
		// Poll every 300ms for new output
		const poll = async () => {
			if (!this.terminals[terminalId]) {
				return; // Terminal was closed
			}
			await this.pollTerminalOutput(terminalId);
			setTimeout(poll, 300);
		};
		
		setTimeout(poll, 300);
	}

	private async pollTerminalOutput(terminalId: string) {
		const terminal = this.terminals[terminalId];
		
		if (!terminal) return;

		try {
			const content = await invoke<string>("get_terminal_content", {
				key: this.projectKey,
				terminalId,
			});

			// Check if there's new content
			if (content.length > terminal.lastProcessedLength) {
				// Get only the new content
				const newContent = content.substring(terminal.lastProcessedLength);
				terminal.rawContent = content;
				console.log("New terminal content:", newContent);
				// Parse new lines and add to history
				const newLines = newContent.split('\n').filter(line => line.trim());
				let lastExecutedCommands = [];
				for (const line of terminal.history.slice().reverse()) {
					if (line.type === "command") {
						lastExecutedCommands.push(line.content.trim());
					} else {
						break;
					}
				}

				for (const line of newLines) {
					if (lastExecutedCommands.includes(line.trim())) {
						// Skip echo of last executed commands
						continue;
					}

					console.log({
						lastExecutedCommands,
						line,
					})
					terminal.history.push({
						type: "output",
						content: line.trim(),
					});
				}

				console.log("Updated terminal history:", terminal.history);

				terminal.lastProcessedLength = content.length;

				// Force update
				this.terminals = { ...this.terminals };
			}
		} catch (error) {
			// Silently fail - terminal might be closing
			console.debug("Failed to poll terminal output:", error);
		}
	}

	async closeTerminal(terminalId: string) {
		try {
			await invoke("close_terminal", {
				key: this.projectKey,
				terminalId,
			});

			delete this.terminals[terminalId];
			
			// Force reactivity update
			this.terminals = { ...this.terminals };
			
			console.log("Terminal closed:", terminalId);
		} catch (error) {
			console.error("Failed to close terminal:", error);
		}
	}

	clearAll() {
		// Close all terminals
		for (const terminalId of Object.keys(this.terminals)) {
			this.closeTerminal(terminalId).catch(console.error);
		}
	}
}

export const terminalManager = new TerminalManager();