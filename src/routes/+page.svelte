<script lang="ts">
  import { storage } from "$lib";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardFooter,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import ResponsiveDialog from "$lib/components/ui/responsive-dialog/responsive-dialog.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getStore } from "@tauri-apps/plugin-store";
  import { toast } from "svelte-sonner";
	import { platform } from "@tauri-apps/plugin-os"

  let platformName = platform();

  let projects = $state(
    [] as {
      name: string;
      host: string;
      user: string;
      password?: string;
      keyFile?: string;
      publicKeyFile?: string;
      authMethod: "password" | "key" | "public_key" | "agent";
    }[],
  );
  let open = $state(false);
  let name = $state("");
  let host = $state("");
  let user = $state("");
  let password = $state("");
  let keyFile = $state("");
  let publicKeyFile = $state("");
  let authMethod = $state<"password" | "key" | "public_key" | "agent">("password");
  let loading = $state(false);

  function nameToKey(name: string) {
    return name.toLowerCase().replace(/\s+/g, "_");
  }

  let statusMap: Record<string, "online" | "offline"> = $state({});

  async function checkProjectStatus(project: {
    name: string;
    host: string;
    user: string;
    password?: string;
    keyFile?: string;
    publicKeyFile?: string;
    authMethod: "password" | "key" | "public_key" | "agent";
  }) {
    try {
      await invoke<boolean>("validate_ssh_connection", {
        host: project.host,
        user: project.user,
        password: project.authMethod === "password" ? project.password : undefined,
        keyFile: project.authMethod === "key" ? project.keyFile : undefined,
        publicKeyFile: project.authMethod === "key" ? project.publicKeyFile : undefined,
        authMethod: project.authMethod,
      });
      statusMap[nameToKey(project.name)] = "online";
    } catch {
      statusMap[nameToKey(project.name)] = "offline";
    }
  }

  $effect(() => {
    (async () => {
      try {
        while (!storage) {
          await new Promise((resolve) => setTimeout(resolve, 100));
        }
        let projectKeys: Array<string> = (await storage.get("projects")) ?? [];
        let loadedProjects = [];
        console.log("Loading projects for keys:");
        console.log(projectKeys);
        for (let key of projectKeys) {
          try {
            if (!key || typeof key !== "string") continue;
            let project = await storage.get(key);
            if (project) {
              loadedProjects.push(project as {
                name: string;
                host: string;
                user: string;
                password?: string;
                keyFile?: string;
                publicKeyFile?: string;
                authMethod: "password" | "key" | "public_key" | "agent";
              });
            }
          } catch (e) {
            console.error("Failed to load project for key:", key, e);
          }
        }
        projects = loadedProjects;
        
        // Check status for all loaded projects
        for (let project of loadedProjects) {
          checkProjectStatus(project);
        }
      } catch (e) {
        toast.error("Failed to load projects: " + String(e));
      }
    })();
  })
  

  async function addProject(event: Event) {
    event.preventDefault();
    loading = true;

    try {
      // Validate SSH connection before adding project
      await invoke<boolean>("validate_ssh_connection", {
        host,
        user,
        password: authMethod === "password" ? password : undefined,
        keyFile: authMethod === "key" ? keyFile : undefined,
        publicKeyFile: authMethod === "key" ? publicKeyFile : undefined,
        authMethod,
      });

      let projectKeys: Array<string> = (await storage.get("projects")) ?? [];
      storage.set("projects", 
        [nameToKey(name), ...projectKeys]
      );

      storage.set(nameToKey(name), {
        name,
        host,
        user,
        password: authMethod === "password" ? password : undefined,
        keyFile: authMethod === "key" ? keyFile : undefined,
        publicKeyFile: authMethod === "key" ? publicKeyFile : undefined,
        authMethod,
      });

      await storage.save();

      const newProject = { name, host, user, password, keyFile, publicKeyFile, authMethod };
      projects.push(newProject);
      
      // Check status for the new project
      checkProjectStatus(newProject);

      toast.success(`Project "${name}" added successfully`);
      open = false;
      name = host = user = password = keyFile = publicKeyFile = "";
      authMethod = "password";
    } catch (err) {
      toast.error("UI: " + String(err));
    } finally {
      loading = false;
    }
  }
  
  $effect(() => {
    if (open && !loading) {
      name = host = user = password = keyFile = publicKeyFile = "";
      authMethod = "password";
    }
  });

  async function connect(project: {
    name: string;
    host: string;
    user: string;
    password?: string;
    keyFile?: string;
    publicKeyFile?: string;
    authMethod: "password" | "key" | "public_key" | "agent";
  }) {
    try {
      const isValid = await invoke<boolean>("validate_ssh_connection", {
        host: project.host,
        user: project.user,
        password: project.authMethod === "password" ? project.password : undefined,
        keyFile: project.authMethod === "key" ? project.keyFile : undefined,
        publicKeyFile: project.authMethod === "key" ? project.publicKeyFile : undefined,
        authMethod: project.authMethod,
      });
      
      if (isValid) {
        // Start the project and open remote window
        await invoke("start_project", {
          key: nameToKey(project.name),
          name: project.name,
          host: project.host,
          user: project.user,
          password: project.password,
          keyFile: project.keyFile,
          publicKeyFile: project.publicKeyFile,
          authMethod: project.authMethod,
        });
        
        toast.success(`Connected to ${project.name}`);
      }
    } catch (err) {
      toast.error(String(err));
    }
  }

  // Snippets for ResponsiveDialog
  function triggerSnippet() {
    return null;
  }

  function childrenSnippet() {
    return null;
  }

</script>

<main class="min-h-screen bg-background">
  <!-- Header -->
  <header class="border-b bg-card">
    <div class="mx-auto max-w-7xl px-4 py-4 sm:px-6 sm:py-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <h1 class="text-xl sm:text-2xl font-semibold tracking-tight">Remote Explorer</h1>
        <p class="text-xs sm:text-sm text-muted-foreground">
          Manage and connect to your remote environments
        </p>
      </div>
      <ResponsiveDialog bind:open title="Add New Project">
        {#snippet trigger()}
          <Button size="sm" class="w-full sm:w-auto">Add Project</Button>
        {/snippet}
        
        <form onsubmit={addProject} class="space-y-4">
            <div class="space-y-1">
              <!-- svelte-ignore a11y_label_has_associated_control -->
              <label class="text-sm font-medium">Project Name</label>
              <Input
                bind:value={name}
                placeholder="My Project"
                required
                disabled={loading}
              />
            </div>
            <div class="space-y-1">
              <!-- svelte-ignore a11y_label_has_associated_control -->
              <label class="text-sm font-medium">Host</label>
              <Input
                bind:value={host}
                placeholder="example.com or 192.168.1.1:2222"
                required
                disabled={loading}
              />
            </div>
            <!-- svelte-ignore a11y_label_has_associated_control -->
            <div class="space-y-1">
              <label class="text-sm font-medium">User</label>
              <Input
                bind:value={user}
                placeholder="username"
                required
                disabled={loading}
              />
            </div>

            <!-- Authentication Method Selection -->
            <div class="space-y-2">
              <!-- svelte-ignore a11y_label_has_associated_control -->
              <label class="text-sm font-medium">Authentication Method</label>
              <div class="grid grid-cols-2 gap-2 sm:flex sm:gap-0">
                <Button
                  type="button"
                  variant={authMethod === "password" ? "default" : "outline"}
                  size="sm"
                  onclick={() => (authMethod = "password")}
                  disabled={loading}
                  class="text-xs sm:text-sm"
                >
                  Password
                </Button>
                <Button
                  type="button"
                  variant={authMethod === "key" ? "default" : "outline"}
                  size="sm"
                  onclick={() => (authMethod = "key")}
                  disabled={loading}
                  class="text-xs sm:text-sm"
                >
                  Private Key
                </Button>
                {#if platformName !== "windows"}
                  <Button
                    type="button"
                    variant={authMethod === "public_key" ? "default" : "outline"}
                    size="sm"
                    onclick={() => (authMethod = "public_key")}
                    disabled={loading}
                    class="text-xs sm:text-sm"
                  >
                    Public Key
                  </Button>
                {/if}
                <Button
                  type="button"
                  variant={authMethod === "agent" ? "default" : "outline"}
                  size="sm"
                  onclick={() => (authMethod = "agent")}
                  disabled={loading}
                  class="text-xs sm:text-sm"
                >
                  SSH Agent
                </Button>
              </div>
            </div>

            <!-- Authentication Fields -->
            {#if authMethod === "password"}
              <div class="space-y-1">
                <!-- svelte-ignore a11y_label_has_associated_control -->
                <label class="text-sm font-medium">Password</label>
                <Input
                  type="password"
                  bind:value={password}
                  placeholder="Enter your password"
                  required
                  disabled={loading}
                />
              </div>
            {:else if authMethod === "key"}
              <div class="space-y-1">
                <!-- svelte-ignore a11y_label_has_associated_control -->
                <label class="text-sm font-medium">Private Key Path</label>
                <Input
                  type="text"
                  placeholder="/home/user/.ssh/id_rsa"
                  bind:value={keyFile}
                  required
                  disabled={loading}
                />
                <p class="text-xs text-muted-foreground">
                  Full path to your private SSH key file
                </p>
              </div>
            {:else if authMethod === "public_key"}
              <div class="space-y-1">
                <!-- svelte-ignore a11y_label_has_associated_control -->
                <label class="text-sm font-medium">Public Key Path</label>
                <Input
                  type="text"
                  placeholder="/home/user/.ssh/id_rsa.pub"
                  bind:value={publicKeyFile}
                  required
                  disabled={loading}
                />
                <p class="text-xs text-muted-foreground">
                  Full path to your public SSH key file
                </p>
              </div>
            {:else}
              <div class="space-y-1">
                <p class="text-sm text-muted-foreground">
                  SSH Agent will be used to authenticate. Make sure your SSH agent is running and has your keys loaded.
                </p>
              </div>
            {/if}

            <div class="flex gap-2">
              <Button type="submit" class="flex-1" disabled={loading}>
                {loading ? "Validating..." : "Create Project"}
              </Button>
            </div>
          </form>
      </ResponsiveDialog>
    </div>
  </header>

  <!-- Content -->
  <section class="mx-auto max-w-7xl px-4 py-6 sm:px-6 sm:py-10">
    {#if projects.length === 0}
      <div
        class="flex flex-col items-center justify-center rounded-lg border border-dashed py-12 sm:py-20 text-center px-4"
      >
        <p class="text-xs sm:text-sm text-muted-foreground">No projects yet</p>
        <p class="mt-1 text-xs sm:text-sm">Create your first remote connection</p>
      </div>
    {:else}
      <div class="grid gap-4 sm:gap-6 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
        {#each projects as project}
          <Card class="transition hover:shadow-md">
            <CardHeader class="pb-2">
              <div class="flex items-center gap-2 min-w-0">
                <div class="relative shrink-0">
                  <div
                    class="h-2.5 w-2.5 rounded-full {statusMap[nameToKey(project.name)] === 'online' ? 'bg-green-500' : 'bg-red-500'}"
                  ></div>
                  <div
                    class="absolute inset-0 h-2.5 w-2.5 rounded-full {statusMap[nameToKey(project.name)] === 'online' ? 'bg-green-500' : 'bg-red-500'} animate-ping opacity-75"
                  ></div>
                </div>
                <CardTitle class="text-base sm:text-lg truncate">{project.name}</CardTitle>
              </div>
            </CardHeader>
            <CardContent class="space-y-1 text-xs sm:text-sm text-muted-foreground">
              <div class="truncate">
                Host: <span class="text-foreground break-all">{project.host}</span>
              </div>
              <div class="truncate">
                User: <span class="text-foreground">{project.user}</span>
              </div>
              <div>
                Auth: <span class="text-foreground">
                  {project.authMethod === "password"
                    ? "Password"
                    : project.authMethod === "key"
                      ? "Private Key"
                      : project.authMethod === "public_key"
                        ? "Public Key"
                        : "SSH Agent"}
                </span>
              </div>
            </CardContent>
            <CardFooter>
              <Button class="w-full text-sm sm:text-base" onclick={() => connect(project)}
                >Connect</Button
              >
            </CardFooter>
          </Card>
        {/each}
      </div>
    {/if}
  </section>
</main>
