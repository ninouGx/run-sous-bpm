<script lang="ts">
  import { onMount } from "svelte";
  import {
    activitiesStore,
    sortedActivities,
    ActivityList,
  } from "$lib/features/activity";
  import { toast } from "svelte-sonner";

  onMount(async () => {
    await activitiesStore.load();
  });

  async function handleSync() {
    try {
      await activitiesStore.sync();
      toast.success("Activities synced successfully!");
    } catch (error: unknown) {
      toast.error(error instanceof Error ? error.message : "Failed to sync activities");
    }
  }
</script>

<div class="space-y-6">
  <h1 class="text-3xl font-bold">Activities</h1>

  <ActivityList
    activities={$sortedActivities}
    isLoading={$activitiesStore.isLoading}
    error={$activitiesStore.error}
    onSync={handleSync}
  />
</div>
