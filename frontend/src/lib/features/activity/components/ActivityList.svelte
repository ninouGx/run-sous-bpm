<script lang="ts">
  import type {
    ActivityStream,
    StravaActivity,
    TrackWithTimestamp,
  } from "$lib/shared/api/types";
  import ActivityCard from "./ActivityCard.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { activityMusicService } from "../activity-music.service";
  import { toast } from "svelte-sonner";
  import { activitiesService } from "$lib/features/activity/activity.service";

  interface Props {
    activities: StravaActivity[];
    isLoading?: boolean;
    error?: string | null;
    onSync?: () => void;
  }

  let { activities, isLoading = false, error = null, onSync }: Props = $props();

  // Track which activity is currently expanded (only one at a time)
  let expandedActivityId = $state<string | null>(null);

  let musicByActivity = $state<Record<string, TrackWithTimestamp[]>>({});
  let isLoadingMusicForActivity = $state<string | null>(null);

  let activityStreamsCache = $state<Record<string, ActivityStream>>({});
  let isLoadingStreamsForActivity = $state<string | null>(null);

  async function handleToggle(activity: StravaActivity) {
    const activityId = activity.id;

    if (expandedActivityId === activityId) {
      expandedActivityId = null; // Collapse if already expanded
    } else {
      expandedActivityId = activityId; // Expand this one

      // Fetch music data if not already loaded
      if (!musicByActivity[activityId]) {
        isLoadingMusicForActivity = activityId;
        try {
          const response =
            await activityMusicService.getActivityMusic(activityId);
          musicByActivity[activityId] = response.tracks;
        } catch (error: unknown) {
          toast.error("Failed to load music data for this activity");
          musicByActivity[activityId] = []; // Set empty array to prevent retrying
        } finally {
          isLoadingMusicForActivity = null;
        }
      }

      // Fetch activity streams if not already loaded
      if (!activityStreamsCache[activityId]) {
        isLoadingStreamsForActivity = activityId;
        try {
          const streams =
            await activitiesService.getActivityStreams(activityId);

          activityStreamsCache[activityId] = streams;
        } catch (syncError: unknown) {
          toast.error("Failed to sync activity data from Strava");
          activityStreamsCache[activityId] = [];
        } finally {
          isLoadingStreamsForActivity = null;
        }
      }
    }
  }
</script>

<div class="space-y-4">
  <!-- Header with sync button -->
  <div class="flex items-center justify-between">
    <h2 class="text-2xl font-bold">Recent Activities</h2>
    {#if onSync}
      <Button variant="outline" size="sm" onclick={onSync} disabled={isLoading}>
        {isLoading ? "Syncing..." : "Sync Activities"}
      </Button>
    {/if}
  </div>

  <!-- Error state -->
  {#if error}
    <div
      class="bg-destructive/10 border border-destructive text-destructive px-4 py-3 rounded"
    >
      {error}
    </div>
  {/if}

  <!-- Loading state -->
  {#if isLoading && activities.length === 0}
    <div class="space-y-4">
      {#each Array(5) as _}
        <Skeleton class="h-24" />
      {/each}
    </div>
  {:else if activities.length === 0}
    <!-- Empty state -->
    <div class="text-center py-12">
      <p class="text-muted-foreground mb-4">
        No activities found. Connect your Strava account and sync your
        activities.
      </p>
      {#if onSync}
        <Button onclick={onSync}>Sync Now</Button>
      {/if}
    </div>
  {:else}
    <!-- Activities list (vertical stack) -->
    <div class="space-y-4">
      {#each activities as activity (activity.id)}
        <ActivityCard
          {activity}
          activityStream={activityStreamsCache[activity.id] ?? []}
          isLoadingActivityStream={isLoadingStreamsForActivity === activity.id}
          musics={musicByActivity[activity.id] ?? []}
          isLoadingMusic={isLoadingMusicForActivity === activity.id}
          isExpanded={expandedActivityId === activity.id}
          onToggle={() => handleToggle(activity)}
        />
      {/each}
    </div>
  {/if}
</div>
