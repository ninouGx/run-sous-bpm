<script lang="ts">
  import type {
    ActivityStream,
    MusicSegment,
    StravaActivity,
    TrackWithTimestamp,
  } from "$lib/shared/api/types";
  import { Card, CardContent } from "$lib/components/ui/card";
  import { Separator } from "$lib/components/ui/separator";
  import {
    formatDistance,
    formatDuration,
    formatActivityDate,
    formatTime,
  } from "../utils/activity-formatters";
  import { ChevronDown, ChevronUp } from "@lucide/svelte";
  import type { Map } from "maplibre-gl";
  import MapTooltip from "./MapTooltip.svelte";
  import type { MapGeoJSONFeature } from "maplibre-gl";
  import { getTrackColor, getTrackColorWithAlpha } from "../utils/track-colors";
  import { createRouteGeoJSON, calculateBounds } from "../utils/geoJsonTransformers";
  import { createTimelineRefs, scrollToTimelineItem } from "../utils/timeline-refs";

  interface Props {
    activity: StravaActivity;
    activityStream: ActivityStream;
    isLoadingActivityStream?: boolean;
    musics: TrackWithTimestamp[];
    segments: MusicSegment[];
    isLoadingMusic?: boolean;
    isExpanded?: boolean;
    onToggle?: () => void;
  }

  let {
    activity,
    activityStream,
    isLoadingActivityStream = false,
    segments,
    musics = extractTracksFromSegments(segments),
    isLoadingMusic = false,
    isExpanded = false,
    onToggle,
  }: Props = $props();

  // Lazy-loaded MapLibre components
  let MapLibre: any = $state(null);
  let NavigationControl: any = $state(null);
  let ScaleControl: any = $state(null);
  let GeoJSONSource: any = $state(null);
  let LineLayer: any = $state(null);
  let FeatureState: any = $state(null);
  let mapComponentsLoaded = $state(false);

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

  // Sort musics by timestamp
  let sortedMusics = $derived(
    [...musics].sort(
      (a, b) =>
        new Date(a.played_at).getTime() - new Date(b.played_at).getTime()
    )
  );

  // Lazy load MapLibre components when expanded
  $effect(() => {
    if (isExpanded && !mapComponentsLoaded) {
      import("svelte-maplibre-gl").then((module) => {
        MapLibre = module.MapLibre;
        NavigationControl = module.NavigationControl;
        ScaleControl = module.ScaleControl;
        GeoJSONSource = module.GeoJSONSource;
        LineLayer = module.LineLayer;
        FeatureState = module.FeatureState;
        mapComponentsLoaded = true;
      });
    }
  });

  let map = $state<Map>();
  let isMapStyleLoaded = $state(false);
  let mapContainer = $state<HTMLDivElement | undefined>(undefined);

  // Cleanup map instance when collapsed
  $effect(() => {
    if (!isExpanded && map) {
      try {
        map.remove();
      } catch (e) {
        // Map already removed, ignore
      }
      map = undefined;
      isMapStyleLoaded = false;
      mapComponentsLoaded = false;
    }
  });

  // Cleanup on component unmount
  $effect(() => {
    return () => {
      if (map) {
        try {
          map.remove();
        } catch (e) {
          // Map already removed, ignore
        }
      }
    };
  });

  // Track hovered feature for map interactions
  let hoveredFeature = $state<MapGeoJSONFeature | undefined>(undefined);
  let hoveredTrackSegmentId = $state<string | null>(null);
  let selectedTrackSegmentId = $state<string | null>(null);
  let tooltipData = $state<{
    trackName: string;
    artistName: string;
    x: number;
    y: number;
  } | null>(null);

  // Timeline refs management
  const { bindTimelineItemRef, timelineItemRefs } = createTimelineRefs();

  // Computed tooltip data that prioritizes hover over selection
  let displayedTooltipData = $derived.by(() => {
    // Priority 1: Show hover tooltip if available
    if (tooltipData) {
      return tooltipData;
    }

    // Priority 2: Show selected segment tooltip at fixed position
    if (selectedTrackSegmentId !== null && mapContainer) {
      const segmentIndex = parseInt(selectedTrackSegmentId);
      const segment = segments[segmentIndex];

      // Only show if track info exists
      if (segment?.track?.track_name && segment?.track?.artist_name) {
        const containerWidth = mapContainer.clientWidth;
        return {
          trackName: segment.track.track_name,
          artistName: segment.track.artist_name,
          x: containerWidth / 2,
          y: 60,
        };
      }
    }

    // No tooltip
    return null;
  });

  // Autoscroll to selected timeline item
  $effect(() => {
    scrollToTimelineItem(timelineItemRefs, selectedTrackSegmentId);
  });

  // Generate GeoJSON for route visualization
  let routeGeoJSONCollection = $derived(createRouteGeoJSON(segments));

  // Calculate map bounds from activity stream
  let bounds = $derived(calculateBounds(activityStream));

  function handleLoad(e: { target: Map }) {
    map = e.target;

    if (map.isStyleLoaded()) {
      isMapStyleLoaded = true;
      fitMapBounds();
    } else {
      map.once("styledata", () => {
        isMapStyleLoaded = true;
        fitMapBounds();
      });
    }
  }

  function fitMapBounds() {
    if (map && bounds) {
      map.fitBounds(
        [
          [bounds.minLng, bounds.minLat],
          [bounds.maxLng, bounds.maxLat],
        ],
        { padding: 50 }
      );
    }
  }
</script>

<Card class="hover:shadow-md transition-shadow">
  <!-- Collapsed Header (always visible) -->
  <button onclick={onToggle} class="w-full text-left">
    <CardContent class="py-4">
      <div class="flex items-center justify-between">
        <div class="flex-1">
          <div class="flex items-center gap-2 mb-1">
            <h3 class="font-semibold text-lg">{activity.name}</h3>
          </div>
          <p class="text-sm text-muted-foreground">
            {formatActivityDate(activity.start_time)} • {formatDistance(
              activity.distance
            )} • {formatDuration(activity.moving_time)} • {activity.type}
          </p>
        </div>

        <div class="flex items-center gap-2">
          {#if isExpanded}
            <ChevronUp class="h-5 w-5 text-muted-foreground" />
          {:else}
            <ChevronDown class="h-5 w-5 text-muted-foreground" />
          {/if}
        </div>
      </div>
    </CardContent>
  </button>

  <!-- Expanded Content (only visible when expanded) -->
  {#if isExpanded}
    <Separator />
    <CardContent class="pt-4 pb-6">
      <div class="grid grid-cols-1 md:grid-cols-5 gap-4">
        <!-- Left: Map Placeholder (60% width = 3/5) -->
        <div class="md:col-span-3">
          <div class="rounded-lg border overflow-hidden" bind:this={mapContainer}>
            {#if isLoadingActivityStream}
              <div
                class="h-[55vh] min-h-[300px] w-full flex items-center justify-center bg-muted"
              >
                <p class="text-sm text-muted-foreground">Loading map...</p>
              </div>
            {:else if activityStream.length > 0}
              {#if mapComponentsLoaded && MapLibre}
                <MapLibre
                  class="h-[55vh] min-h-[300px] w-full"
                  style="https://basemaps.cartocdn.com/gl/voyager-gl-style/style.json"
                  zoom={12.5}
                  minZoom={8}
                  maxZoom={18}
                  fadeDuration={0}
                  preserveDrawingBuffer={false}
                  onload={handleLoad}
                  onclick={(e: { features?: MapGeoJSONFeature[] }) => {
                    // Deselect when clicking empty map space
                    if (!e.features || e.features.length === 0) {
                      selectedTrackSegmentId = null;
                    }
                  }}
                  center={{
                    lng: activityStream[0].longitude,
                    lat: activityStream[0].latitude,
                  }}
                >
                  <NavigationControl />
                  <ScaleControl />

                  {#if isMapStyleLoaded}
                    <GeoJSONSource
                      id="route"
                      data={routeGeoJSONCollection}
                      promoteId="segment_id"
                    >
                      <!-- Invisible wide hit target layer for easier hover detection -->
                      <LineLayer
                        id="route-hit-target"
                        onclick={(e: {
                          lngLat: any;
                          point: { x: number; y: number };
                          features?: MapGeoJSONFeature[];
                        }) => {
                          const features = e.features;
                          if (features && features.length > 0) {
                            const feature = features[0];
                            const props = feature.properties;
                            const segmentId = props?.segment_id?.toString() ?? null;

                            // Toggle selection: if clicking same segment, deselect
                            if (selectedTrackSegmentId === segmentId) {
                              selectedTrackSegmentId = null;
                            } else {
                              selectedTrackSegmentId = segmentId;
                            }
                          }
                        }}
                        onmousemove={(e: {
                          lngLat: any;
                          point: { x: number; y: number };
                          features?: MapGeoJSONFeature[];
                        }) => {
                          const features = e.features;
                          if (features && features.length > 0) {
                            const feature = features[0];
                            hoveredFeature = feature;

                            const props = feature.properties;
                            const segmentId = props?.segment_id;
                            hoveredTrackSegmentId = segmentId?.toString() ?? null;

                            // Show tooltip if track info exists
                            if (props?.track_name && props?.artist_name) {
                              tooltipData = {
                                trackName: props.track_name,
                                artistName: props.artist_name,
                                x: e.point.x,
                                y: e.point.y,
                              };
                            }
                          }
                        }}
                        onmouseleave={() => {
                          hoveredFeature = undefined;
                          hoveredTrackSegmentId = null;
                          tooltipData = null;
                        }}
                        paint={{
                          "line-color": ["get", "color"],
                          "line-width": 20,
                          "line-opacity": 0,
                        }}
                        layout={{
                          "line-cap": "round",
                          "line-join": "round",
                        }}
                      />

                      <!-- Glow layer for hover and selected effect -->
                      <LineLayer
                        id="route-glow"
                        beforeId="route-line"
                        paint={{
                          "line-color": ["get", "color"],
                          "line-width": [
                            "case",
                            [
                              "any",
                              ["boolean", ["feature-state", "hover"], false],
                              ["boolean", ["feature-state", "selected"], false],
                            ],
                            10,
                            0,
                          ],
                          "line-opacity": [
                            "case",
                            [
                              "any",
                              ["boolean", ["feature-state", "hover"], false],
                              ["boolean", ["feature-state", "selected"], false],
                            ],
                            0.3,
                            0,
                          ],
                          "line-blur": 4,
                        }}
                        layout={{
                          "line-cap": "round",
                          "line-join": "round",
                        }}
                      />
                      <!-- Main route line (visual only, no hover handlers) -->
                      <LineLayer
                        id="route-line"
                        paint={{
                          "line-color": ["get", "color"],
                          "line-width": [
                            "case",
                            [
                              "any",
                              ["boolean", ["feature-state", "hover"], false],
                              ["boolean", ["feature-state", "selected"], false],
                            ],
                            6,
                            3,
                          ],
                          "line-opacity": [
                            "case",
                            [
                              "any",
                              ["boolean", ["feature-state", "hover"], false],
                              ["boolean", ["feature-state", "selected"], false],
                            ],
                            1.0,
                            0.5,
                          ],
                          "line-blur": [
                            "case",
                            [
                              "any",
                              ["boolean", ["feature-state", "hover"], false],
                              ["boolean", ["feature-state", "selected"], false],
                            ],
                            0,
                            0.5,
                          ],
                        }}
                        layout={{
                          "line-cap": "round",
                          "line-join": "round",
                        }}
                      />

                      <!-- Declarative hover state management -->
                      {#if hoveredFeature && hoveredFeature.id !== undefined}
                        <FeatureState
                          source="route"
                          id={hoveredFeature.id}
                          state={{ hover: true }}
                        />
                      {:else if hoveredTrackSegmentId !== null}
                        <FeatureState
                          source="route"
                          id={parseInt(hoveredTrackSegmentId)}
                          state={{ hover: true }}
                        />
                      {/if}

                      <!-- Declarative selected state management -->
                      {#if selectedTrackSegmentId !== null}
                        <FeatureState
                          source="route"
                          id={parseInt(selectedTrackSegmentId)}
                          state={{ selected: true }}
                        />
                      {/if}
                    </GeoJSONSource>
                  {/if}

                  <!-- Tooltip for hover or selected -->
                  {#if displayedTooltipData}
                    <MapTooltip
                      trackName={displayedTooltipData.trackName}
                      artistName={displayedTooltipData.artistName}
                      x={displayedTooltipData.x}
                      y={displayedTooltipData.y}
                    />
                  {/if}
                </MapLibre>
              {:else}
                <div
                  class="h-[55vh] min-h-[300px] w-full flex items-center justify-center bg-muted"
                >
                  <p class="text-sm text-muted-foreground">
                    Loading map components...
                  </p>
                </div>
              {/if}
            {:else}
              <div
                class="h-[55vh] min-h-[300px] w-full flex items-center justify-center bg-muted"
              >
                <p class="text-sm text-muted-foreground">
                  No map data available
                </p>
              </div>
            {/if}
          </div>
        </div>

        <!-- Right: Music Timeline (40% width = 2/5) -->
        <div class="md:col-span-2">
          <div class="space-y-3">
            <div class="flex items-center gap-2 mb-4">
              <h4 class="font-semibold text-sm">🎵 Music Timeline</h4>
            </div>

            <div class="space-y-2 max-h-96 overflow-y-auto pr-2" role="list">
              {#if isLoadingMusic}
                <p class="text-sm text-muted-foreground">Loading music...</p>
              {:else if sortedMusics.length === 0}
                <p class="text-sm text-muted-foreground">
                  No music data available for this activity.
                </p>
              {:else}
                {#each sortedMusics as track}
                  {@const segmentIndex = segments.findIndex(
                    (seg) => seg.track?.id === track.track_id
                  )}
                  {@const isHovered =
                    hoveredTrackSegmentId === segmentIndex.toString()}
                  {@const isSelected =
                    selectedTrackSegmentId === segmentIndex.toString()}
                  {@const segmentId = segmentIndex.toString()}

                  <div
                    class="flex items-start gap-3 rounded-md px-3 py-2 transition-colors cursor-pointer
                           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring
                           {isHovered || isSelected ? 'bg-accent' : 'hover:bg-accent/50'}"
                    role="button"
                    tabindex="0"
                    use:bindTimelineItemRef={segmentId}
                    onclick={() => {
                      const segmentId = segmentIndex.toString();
                      // Toggle selection: if clicking same segment, deselect
                      if (selectedTrackSegmentId === segmentId) {
                        selectedTrackSegmentId = null;
                      } else {
                        selectedTrackSegmentId = segmentId;
                      }
                    }}
                    onmouseenter={() =>
                      (hoveredTrackSegmentId = segmentIndex.toString())}
                    onmouseleave={() => (hoveredTrackSegmentId = null)}
                    onfocus={() =>
                      (hoveredTrackSegmentId = segmentIndex.toString())}
                    onblur={() => (hoveredTrackSegmentId = null)}
                    onkeydown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        const segmentId = segmentIndex.toString();
                        // Toggle selection: if clicking same segment, deselect
                        if (selectedTrackSegmentId === segmentId) {
                          selectedTrackSegmentId = null;
                        } else {
                          selectedTrackSegmentId = segmentId;
                        }
                      } else if (e.key === "ArrowDown") {
                        e.preventDefault();
                        const next = e.currentTarget
                          .nextElementSibling as HTMLElement;
                        next?.focus();
                      } else if (e.key === "ArrowUp") {
                        e.preventDefault();
                        const prev = e.currentTarget
                          .previousElementSibling as HTMLElement;
                        prev?.focus();
                      }
                    }}
                  >
                    <!-- Color bar indicator -->
                    <div
                      class="w-1 rounded-full transition-all"
                      style="
                        background-color: {getTrackColor(segmentIndex)};
                        height: {isHovered || isSelected ? '100%' : '50%'};
                        min-height: {isHovered || isSelected ? '40px' : '24px'};
                      "
                    ></div>

                    <div class="min-w-0 flex-1">
                      <p class="font-medium truncate" title={track.track_name}>
                        {track.track_name}
                      </p>
                      <p
                        class="text-sm text-muted-foreground truncate"
                        title={track.artist_name}
                      >
                        {track.artist_name}
                      </p>
                    </div>
                    <p
                      class="text-sm text-muted-foreground whitespace-nowrap flex-shrink-0"
                      title={track.played_at}
                    >
                      {formatTime(track.played_at)}
                    </p>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        </div>
      </div>
    </CardContent>
  {/if}
</Card>
