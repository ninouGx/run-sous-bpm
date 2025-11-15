<script lang="ts">
  import { userStore } from "$lib/stores/user";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  // Si déjà connecté, redirect vers dashboard
  onMount(() => {
    if ($userStore.isAuthenticated) {
      goto("/dashboard");
    }
  });
</script>

<div
  class="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-50 to-slate-100"
>
  <div class="w-full max-w-md p-8">
    <slot />
  </div>
</div>
