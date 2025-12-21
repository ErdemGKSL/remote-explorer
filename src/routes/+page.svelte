<script lang="ts">
    import { storage } from "$lib";
  import { Button } from "$lib/components/ui/button";
  import * as ButtonGroup from "$lib/components/ui/button-group";
  import {
    Card,
    CardContent,
    CardFooter,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import {
    Dialog,
    DialogContent,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
  } from "$lib/components/ui/dialog";
  import { invoke } from "@tauri-apps/api/core";
    import { getStore } from "@tauri-apps/plugin-store";
  import { toast } from "svelte-sonner";

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

  $effect(() => {
    (async () => {
      try {
        let projectKeys: Array<string> = (await storage.get("projects")) ?? [];
        let loadedProjects = [];
        for (let key of projectKeys) {
          try {
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
          } catch {}
        }
        projects = loadedProjects;
      } catch {}
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
      storage.set("projects", [
        [nameToKey(name), ...projectKeys]
      ]);

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

      projects.push({ name, host, user, password, keyFile, publicKeyFile, authMethod });

      toast.success(`Project "${name}" added successfully`);
      open = false;
      name = host = user = password = keyFile = publicKeyFile = "";
      authMethod = "password";
    } catch (err) {
      toast.error(String(err));
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
        toast.success(`Connected to ${project.name}`);
        console.log("Connected to", project);
      }
    } catch (err) {
      toast.error(String(err));
    }
  }
</script>

<main class="min-h-screen bg-background">
  <!-- Header -->
  <header class="border-b bg-card">
    <div class="mx-auto max-w-7xl px-6 py-6 flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-semibold tracking-tight">Remote Explorer</h1>
        <p class="text-sm text-muted-foreground">
          Manage and connect to your remote environments
        </p>
      </div>
      <Dialog bind:open>
        <DialogTrigger>
          <Button size="sm">Add Project</Button>
        </DialogTrigger>
        <DialogContent class="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Add New Project</DialogTitle>
          </DialogHeader>
          <form onsubmit={addProject} class="space-y-4">
            <div class="space-y-1">
              <!-- svelte-ignore a11y_label_has_associated_control -->
              <label class="text-sm font-medium">Project Name</label>
              <input
                class="w-full rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                bind:value={name}
                required
                disabled={loading}
              />
            </div>
            <div class="space-y-1">
              <!-- svelte-ignore a11y_label_has_associated_control -->
              <label class="text-sm font-medium">Host</label>
              <input
                class="w-full rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                bind:value={host}
                required
                disabled={loading}
              />
            </div>
            <!-- svelte-ignore a11y_label_has_associated_control -->
            <div class="space-y-1">
              <label class="text-sm font-medium">User</label>
              <input
                class="w-full rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                bind:value={user}
                required
                disabled={loading}
              />
            </div>

            <!-- Authentication Method Selection -->
            <div class="space-y-2">
              <!-- svelte-ignore a11y_label_has_associated_control -->
              <label class="text-sm font-medium">Authentication Method</label>
              <ButtonGroup.Root>
                <Button
                  type="button"
                  variant={authMethod === "password" ? "default" : "outline"}
                  size="sm"
                  onclick={() => (authMethod = "password")}
                  disabled={loading}
                >
                  Password
                </Button>
                <Button
                  type="button"
                  variant={authMethod === "key" ? "default" : "outline"}
                  size="sm"
                  onclick={() => (authMethod = "key")}
                  disabled={loading}
                >
                  Private Key
                </Button>
                <Button
                  type="button"
                  variant={authMethod === "public_key" ? "default" : "outline"}
                  size="sm"
                  onclick={() => (authMethod = "public_key")}
                  disabled={loading}
                >
                  Public Key
                </Button>
                <Button
                  type="button"
                  variant={authMethod === "agent" ? "default" : "outline"}
                  size="sm"
                  onclick={() => (authMethod = "agent")}
                  disabled={loading}
                >
                  SSH Agent
                </Button>
              </ButtonGroup.Root>
            </div>

            <!-- Authentication Fields -->
            {#if authMethod === "password"}
              <div class="space-y-1">
                <!-- svelte-ignore a11y_label_has_associated_control -->
                <label class="text-sm font-medium">Password</label>
                <input
                  type="password"
                  class="w-full rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
                  bind:value={password}
                  required
                  disabled={loading}
                />
              </div>
            {:else if authMethod === "key"}
              <div class="space-y-1">
                <!-- svelte-ignore a11y_label_has_associated_control -->
                <label class="text-sm font-medium">Private Key Path</label>
                <input
                  type="text"
                  placeholder="/home/user/.ssh/id_rsa"
                  class="w-full rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
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
                <input
                  type="text"
                  placeholder="/home/user/.ssh/id_rsa.pub"
                  class="w-full rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
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

            <DialogFooter>
              <Button type="submit" class="w-full" disabled={loading}>
                {loading ? "Validating..." : "Create Project"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  </header>

  <!-- Content -->
  <section class="mx-auto max-w-7xl px-6 py-10">
    {#if projects.length === 0}
      <div
        class="flex flex-col items-center justify-center rounded-lg border border-dashed py-20 text-center"
      >
        <p class="text-sm text-muted-foreground">No projects yet</p>
        <p class="mt-1 text-sm">Create your first remote connection</p>
      </div>
    {:else}
      <div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
        {#each projects as project}
          <Card class="transition hover:shadow-md">
            <CardHeader class="pb-2">
              <CardTitle class="text-lg">{project.name}</CardTitle>
            </CardHeader>
            <CardContent class="space-y-1 text-sm text-muted-foreground">
              <div>
                Host: <span class="text-foreground">{project.host}</span>
              </div>
              <div>
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
              <Button class="w-full" onclick={() => connect(project)}
                >Connect</Button
              >
            </CardFooter>
          </Card>
        {/each}
      </div>
    {/if}
  </section>
</main>
