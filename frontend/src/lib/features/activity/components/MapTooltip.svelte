<script lang="ts">
  interface Props {
    trackName: string;
    artistName: string;
    x: number;
    y: number;
  }

  let { trackName, artistName, x, y }: Props = $props();
</script>

<div
  class="absolute pointer-events-none z-10 bg-popover text-popover-foreground px-3 py-2 rounded-md shadow-md border text-sm"
  style="left: {x}px; top: {y}px; transform: translate(-50%, -120%);"
>
  <p class="font-medium">{trackName}</p>
  <p class="text-muted-foreground text-xs">{artistName}</p>
</div>
