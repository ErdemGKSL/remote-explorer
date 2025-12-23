# Remote Explorer

A cross-platform desktop application for exploring and managing remote file systems via SSH connections. Built with Tauri, SvelteKit, and Rust, this app provides a native file explorer experience for remote servers.

## Features

- **SSH Connection Management**: Securely connect to remote servers using SSH with support for password authentication
- **File System Exploration**: Browse remote directories with a familiar file explorer interface
- **Connection Caching**: Save and reuse SSH connection details for quick access to frequently used servers
- **Real-time File Listing**: Execute `ls -la` commands to fetch and display directory contents
- **Cross-Platform**: Runs on Windows, macOS, Linux and Android
- **Modern UI**: Built with SvelteKit and styled with Tailwind CSS for a responsive and intuitive interface

## Tech Stack

- **Frontend**: SvelteKit, TypeScript, Tailwind CSS
- **Backend**: Tauri (Rust) with async-ssh2-tokio for SSH operations
- **UI Components**: Bits UI, Lucide icons, Shadcn-svelte
- **Build Tool**: Vite
- **Plugins**: Tauri plugins for clipboard, opener, OS integration, and persistent storage

## Prerequisites

Before running this application, ensure you have the following installed:

- [Bun](https://bun.sh/)
- [Rust](https://rustup.rs/) (latest stable version)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/remote-explorer.git
   cd remote-explorer
   ```

2. Install dependencies:
   ```bash
   bun install
   ```

3. Run the application in development mode:
   ```bash
   bun run tauri dev
   ```

This will start the Tauri development server, which will build both the Rust backend and the Svelte frontend.

## Usage

1. **Launch the App**: Run `bun run tauri dev` to start the application.

2. **Main Window**:
   - Select from saved projects/connections
   - Or choose to connect to a new remote server

3. **Project Page**:
   - Enter SSH connection details (host, username, password)
   - The app establishes and caches the SSH connection
   - Browse the remote file system like a local explorer
   - File data is fetched using `ls -la` commands executed via SSH

4. **Navigation**: Click on folders to navigate deeper into the directory structure.

## Development

### Frontend Development
To work on the SvelteKit frontend only:
```bash
bun run dev
```

### Full Application Development
To work on both frontend and backend:
```bash
bun run tauri dev
```

### Building for Production
```bash
bun run tauri build
```

### Type Checking
```bash
bun run check
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) for the desktop application framework
- [SvelteKit](https://kit.svelte.dev/) for the frontend framework
- [async-ssh2-tokio](https://github.com/Miyoshi-Ryota/async-ssh2-tokio) for SSH functionality
