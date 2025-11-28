<script lang="ts">
  import type {
    ActivityStream,
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

  interface Props {
    activity: StravaActivity;
    activityStream: ActivityStream;
    isLoadingActivityStream?: boolean;
    musics: TrackWithTimestamp[];
    isLoadingMusic?: boolean;
    isExpanded?: boolean;
    onToggle?: () => void;
  }

  let {
    activity,
    activityStream,
    isLoadingActivityStream = false,
    musics,
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
  let mapComponentsLoaded = $state(false);

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
        mapComponentsLoaded = true;
      });
    }
  });

  let map = $state<Map>();
  let isMapStyleLoaded = $state(false);

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

  let simplifiedStream = $derived.by(() => {
    // Backend handles simplification via ?simplify=true query param
    // Just filter for valid GPS coordinates
    return activityStream.filter(
      (point) => point.latitude && point.longitude
    );
  });

  let routeGeoJSON = $derived({
    type: "Feature",
    geometry: {
      type: "LineString",
      coordinates: simplifiedStream.map((point) => [
        point.longitude,
        point.latitude,
      ]),
    },
    properties: {},
  } as const);

  let bounds = $derived.by(() => {
    if (simplifiedStream.length === 0) return null;

    const lngs = simplifiedStream.map((p) => p.longitude);
    const lats = simplifiedStream.map((p) => p.latitude);

    return {
      minLng: Math.min(...lngs),
      maxLng: Math.max(...lngs),
      minLat: Math.min(...lats),
      maxLat: Math.max(...lats),
    };
  });

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
          <div class="rounded-lg border overflow-hidden">
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
                  center={{
                    lng: activityStream[0].longitude,
                    lat: activityStream[0].latitude,
                  }}
                >
                  <NavigationControl />
                  <ScaleControl />

                  {#if isMapStyleLoaded}
                    <GeoJSONSource id="route" data={routeGeoJSON}>
                      <LineLayer
                        id="route-line"
                        paint={{
                          "line-color": "#FF5500",
                          "line-width": 3,
                          "line-opacity": 0.8,
                        }}
                      />
                    </GeoJSONSource>
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

            <div class="space-y-2 max-h-96 overflow-y-auto pr-2">
              {#if isLoadingMusic}
                <p class="text-sm text-muted-foreground">Loading music...</p>
              {:else if sortedMusics.length === 0}
                <p class="text-sm text-muted-foreground">
                  No music data available for this activity.
                </p>
              {:else}
                {#each sortedMusics as track}
                  <div class="flex items-center gap-3">
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
