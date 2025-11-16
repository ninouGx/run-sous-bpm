<script lang="ts">
  import { userStore } from "$lib/stores/user";
  import { authService } from "$lib/features/auth/auth.service";
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import { toast } from "svelte-sonner";
  import { Toaster } from "$lib/components/ui/sonner";
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();
  let isLoggingOut = $state(false);

  async function handleLogout() {
    if (isLoggingOut) return;

    isLoggingOut = true;

    try {
      await authService.logout();
      toast.success("Logged out successfully");
      goto("/auth/login");
    } catch (err) {
      toast.error("Failed to logout. Please try again.");
      isLoggingOut = false;
    }
  }
</script>

<div class="min-h-screen bg-background">
  <nav class="border-b">
    <div class="container mx-auto px-4 py-4 flex items-center justify-between">
      <div class="flex items-center gap-8">
        <a href="/" class="text-xl font-bold">Run Sous BPM</a>
      </div>

      <div class="flex items-center gap-4">
        {#if $userStore.user}
          <span class="text-sm text-muted-foreground"
            >{$userStore.user.email}</span
          >
        {/if}
        <Button
          variant="outline"
          size="sm"
          onclick={handleLogout}
          disabled={isLoggingOut}
        >
          {isLoggingOut ? "Logging out..." : "Logout"}
        </Button>
      </div>
    </div>
  </nav>

  <main class="container mx-auto px-4 py-8">
    {@render children()}
  </main>
</div>

<Toaster />
