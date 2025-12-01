<script lang="ts">
  import type { TrackWithTimestamp } from "$lib/shared/api/types";
  import type { Action } from "svelte/action";
  import { formatTime } from "../../utils/activity-formatters";
  import type { Snippet } from "svelte";

  interface Props {
    track: TrackWithTimestamp;
    segmentIndex: number;
    segmentId: string;
    color: string;
    isHovered: boolean;
    isSelected: boolean;
    onSelect: (segmentId: string) => void;
    onHover: (segmentId: string | null) => void;
    bindRef?: Action<HTMLElement, string>;
  }

  let {
    track,
    segmentIndex,
    segmentId,
    color,
    isHovered,
    isSelected,
    onSelect,
    onHover,
    bindRef,
  }: Props = $props();

  function handleClick() {
    onSelect(segmentId);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onSelect(segmentId);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      const next = (e.currentTarget as HTMLElement)
        .nextElementSibling as HTMLElement;
      next?.focus();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      const prev = (e.currentTarget as HTMLElement)
        .previousElementSibling as HTMLElement;
      prev?.focus();
    }
  }
</script>

{#snippet itemContent()}
  <!-- Color bar indicator -->
  <div
    class="w-1 rounded-full transition-all"
    style="
      background-color: {color};
      height: {isHovered || isSelected ? '100%' : '50%'};
      min-height: {isHovered || isSelected ? '40px' : '24px'};
    "
  ></div>

  <div class="min-w-0 flex-1">
    <p class="font-medium truncate" title={track.track_name}>
      {track.track_name}
    </p>
    <p class="text-sm text-muted-foreground truncate" title={track.artist_name}>
      {track.artist_name}
    </p>
  </div>

  <p
    class="text-sm text-muted-foreground whitespace-nowrap flex-shrink-0"
    title={track.played_at}
  >
    {formatTime(track.played_at)}
  </p>
{/snippet}

{#if bindRef}
  <div
    class="flex items-start gap-3 rounded-md px-3 py-2 transition-colors cursor-pointer
           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring
           {isHovered || isSelected ? 'bg-accent' : 'hover:bg-accent/50'}"
    role="button"
    tabindex="0"
    use:bindRef={segmentId}
    onclick={handleClick}
    onmouseenter={() => onHover(segmentId)}
    onmouseleave={() => onHover(null)}
    onfocus={() => onHover(segmentId)}
    onblur={() => onHover(null)}
    onkeydown={handleKeydown}
  >
    {@render itemContent()}
  </div>
{:else}
  <div
    class="flex items-start gap-3 rounded-md px-3 py-2 transition-colors cursor-pointer
           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring
           {isHovered || isSelected ? 'bg-accent' : 'hover:bg-accent/50'}"
    role="button"
    tabindex="0"
    onclick={handleClick}
    onmouseenter={() => onHover(segmentId)}
    onmouseleave={() => onHover(null)}
    onfocus={() => onHover(segmentId)}
    onblur={() => onHover(null)}
    onkeydown={handleKeydown}
  >
    {@render itemContent()}
  </div>
{/if}
