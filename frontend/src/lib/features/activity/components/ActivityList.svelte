<script lang="ts">
  import type {
    ActivityStream,
    ActivityStreamPoint,
    StravaActivity,
    ActivityMusicResponse,
    MusicSegment,
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

  // Store full music segment data from new API
  let musicSegmentsByActivity = $state<Record<string, ActivityMusicResponse>>(
    {}
  );
  let isLoadingMusicForActivity = $state<string | null>(null);

  let activityStreamsCache = $state<Record<string, ActivityStream>>({});

  // TODO: Remove this temporary adapter once UI is updated to use segments directly
  function extractTracksFromSegments(segments: MusicSegment[]) {
    return segments
      .filter((segment) => segment.track)
      .map((segment) => ({
        played_at: segment.start_time,
        track_name: segment.track!.track_name,
        artist_name: segment.track!.artist_name,
        album_name: segment.track?.album_name,
        track_id: segment.track!.id,
        listen_id: segment.track!.id, // Using track ID as listen ID for now
      }));
  }

  // Flatten all GPS points from segments into a single ActivityStream array
  function flattenSegmentPointsToStream(
    activityId: string,
    segments: MusicSegment[]
  ): ActivityStream {
    const allPoints: ActivityStreamPoint[] = [];

    for (const segment of segments) {
      for (const point of segment.points) {
        // Validate GPS coordinates before adding to map
        if (
          typeof point.latitude === 'number' &&
          typeof point.longitude === 'number' &&
          !isNaN(point.latitude) &&
          !isNaN(point.longitude) &&
          Math.abs(point.latitude) <= 90 &&
          Math.abs(point.longitude) <= 180
        ) {
          allPoints.push({
            activity_id: activityId,
            time: point.time,
            distance: 0, // Not available in segments, not critical for map
            latitude: point.latitude,
            longitude: point.longitude,
            altitude: point.altitude ?? 0,
            velocity: point.velocity ?? 0,
            heart_rate: point.heart_rate ?? null,
            cadence: point.cadence ?? null,
            watts: point.watts ?? null,
            temperature: null, // Not available in segments
          });
        } else {
          console.warn('Invalid GPS point filtered out:', {
            segment: segment.index,
            lat: point.latitude,
            lng: point.longitude
          });
        }
      }
    }

    return allPoints;
  }

  async function handleToggle(activity: StravaActivity) {
    const activityId = activity.id;

    if (expandedActivityId === activityId) {
      expandedActivityId = null; // Collapse if already expanded
    } else {
      expandedActivityId = activityId; // Expand this one

      // Fetch music data if not already loaded (includes GPS points)
      if (!musicSegmentsByActivity[activityId]) {
        isLoadingMusicForActivity = activityId;
        try {
          const response =
            await activityMusicService.getActivityMusic(activityId);
          musicSegmentsByActivity[activityId] = response;

          // Extract GPS stream from segments instead of making separate API call
          activityStreamsCache[activityId] = flattenSegmentPointsToStream(
            activityId,
            response.segments
          );

          console.log("Activity music segments loaded:", {
            activity_id: response.activity_id,
            has_gps: response.has_gps,
            segments: response.segments.length,
            gps_points: activityStreamsCache[activityId].length,
            stats: response.stats,
          });
        } catch (error: unknown) {
          toast.error("Failed to load music data for this activity");
          // Set empty response to prevent retrying
          musicSegmentsByActivity[activityId] = {
            activity_id: activityId,
            has_gps: false,
            segments: [],
            stats: {
              total_segments: 0,
              segments_with_music: 0,
              segments_without_music: 0,
              original_points: 0,
              simplified_points: 0,
              reduction_ratio: 0,
            },
          };
          activityStreamsCache[activityId] = [];
        } finally {
          isLoadingMusicForActivity = null;
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
          isLoadingActivityStream={isLoadingMusicForActivity === activity.id}
          musics={musicSegmentsByActivity[activity.id]
            ? extractTracksFromSegments(
                musicSegmentsByActivity[activity.id].segments
              )
            : []}
          isLoadingMusic={isLoadingMusicForActivity === activity.id}
          isExpanded={expandedActivityId === activity.id}
          onToggle={() => handleToggle(activity)}
        />
      {/each}
    </div>
  {/if}
</div>
