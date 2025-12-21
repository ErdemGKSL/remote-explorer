import { invoke } from '@tauri-apps/api/core';

interface TerminalConnection {
    id: string
    connection: any
    name: string
    path: string // this is a start path, if user changes it by cd command, this path is not updated
}

interface ProjectInfo {
    key: string
    name: string
    host: string
    auth_method: string
}

let connection = $state({
    key: "",
    name: "",
    host: "",
    password: null,
    key_file: null,
    public_key_file: null,
    auth_method: "",
    main_connection: null,
    terminal_connections: [] as TerminalConnection[],
});

export async function loadConnection(projectKey: string) {
    try {
        const project = await invoke<ProjectInfo | null>('get_project', { key: projectKey });
        
        if (project) {
            connection.key = project.key;
            connection.name = project.name;
            connection.host = project.host;
            connection.auth_method = project.auth_method;
        } else {
            connection = {
                key: "",
                name: "",
                host: "",
                password: null,
                key_file: null,
                public_key_file: null,
                auth_method: "",
                main_connection: null,
                terminal_connections: [],
            }
            throw new Error('Project not found');
        }
    } catch (e) {
        console.error('Failed to load connection:', e);
        throw e;
    }
}

export function getConnection() {
    return connection;
}