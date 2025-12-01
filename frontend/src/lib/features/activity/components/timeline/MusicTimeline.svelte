<script lang="ts">
  import type {
    TrackWithTimestamp,
    MusicSegment,
  } from "$lib/shared/api/types";
  import type { MapInteractionState } from "../../state/mapInteractionState.svelte";
  import type { Action } from "svelte/action";
  import TimelineItem from "./TimelineItem.svelte";
  import { getTrackColor } from "../../utils/track-colors";

  interface Props {
    tracks: TrackWithTimestamp[];
    segments: MusicSegment[];
    isLoading: boolean;
    interactionState: MapInteractionState;
    bindTimelineItemRef?: Action<HTMLElement, string>;
  }

  let {
    tracks,
    segments,
    isLoading,
    interactionState,
    bindTimelineItemRef,
  }: Props = $props();
</script>

<div class="space-y-3">
  <div class="flex items-center gap-2 mb-4">
    <h4 class="font-semibold text-sm">🎵 Music Timeline</h4>
  </div>

  <div class="space-y-2 max-h-96 overflow-y-auto pr-2" role="list">
    {#if isLoading}
      <p class="text-sm text-muted-foreground">Loading music...</p>
    {:else if tracks.length === 0}
      <p class="text-sm text-muted-foreground">
        No music data available for this activity.
      </p>
    {:else}
      {#each tracks as track}
        {@const segmentIndex = segments.findIndex(
          (seg) => seg.track?.id === track.track_id,
        )}
        {@const isHovered =
          interactionState.hoveredSegmentId === segmentIndex.toString()}
        {@const isSelected =
          interactionState.selectedSegmentId === segmentIndex.toString()}
        {@const segmentId = segmentIndex.toString()}
        {@const color = getTrackColor(segmentIndex)}

        <TimelineItem
          {track}
          {segmentIndex}
          {segmentId}
          {color}
          {isHovered}
          {isSelected}
          onSelect={(id: string) => interactionState.selectSegment(id)}
          onHover={(id: string | null) => interactionState.hoverSegment(id)}
          bindRef={bindTimelineItemRef}
        />
      {/each}
    {/if}
  </div>
</div>
