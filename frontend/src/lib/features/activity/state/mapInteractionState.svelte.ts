/**
 * Centralized state management for map and timeline interactions.
 * This module provides a single source of truth for hover, selection, and tooltip state,
 * enabling synchronized interactions between the map visualization and music timeline.
 */

import type { MapGeoJSONFeature } from "maplibre-gl";
import type { MusicSegment } from "$lib/shared/api/types";

/**
 * Tooltip data structure for displaying track information.
 */
export interface TooltipData {
  trackName: string;
  artistName: string;
  x: number;
  y: number;
}

/**
 * Creates a reactive state object for managing map and timeline interactions.
 * Uses Svelte 5 runes ($state, $derived) for fine-grained reactivity.
 *
 * @param segments - Music segments for tooltip data lookup
 * @param mapContainer - Map container element for positioning tooltips
 * @returns State object with getters, setters, and methods
 *
 * @example
 * ```svelte
 * <script>
 *   const interactionState = createMapInteractionState(segments, mapContainer);
 *
 *   // Select a segment
 *   interactionState.selectSegment('2');
 *
 *   // Access tooltip data
 *   const tooltip = interactionState.displayedTooltip;
 * </script>
 * ```
 */
export function createMapInteractionState(
  segments: MusicSegment[],
  mapContainer: HTMLDivElement | undefined,
) {
  // Hover state
  let hoveredFeature = $state<MapGeoJSONFeature | undefined>(undefined);
  let hoveredSegmentId = $state<string | null>(null);

  // Selection state
  let selectedSegmentId = $state<string | null>(null);

  // Tooltip state (for hover)
  let tooltipData = $state<TooltipData | null>(null);

  /**
   * Computed tooltip data that prioritizes hover over selection.
   * Shows hover tooltip immediately, or selected segment tooltip at fixed position.
   */
  const displayedTooltip = $derived.by(() => {
    // Priority 1: Show hover tooltip if available
    if (tooltipData) {
      return tooltipData;
    }

    // Priority 2: Show selected segment tooltip at fixed position
    if (selectedSegmentId !== null && mapContainer) {
      const segmentIndex = parseInt(selectedSegmentId);
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

  return {
    // Hover state
    get hoveredFeature() {
      return hoveredFeature;
    },
    set hoveredFeature(value) {
      hoveredFeature = value;
    },

    get hoveredSegmentId() {
      return hoveredSegmentId;
    },
    set hoveredSegmentId(value) {
      hoveredSegmentId = value;
    },

    // Selection state
    get selectedSegmentId() {
      return selectedSegmentId;
    },
    set selectedSegmentId(value) {
      selectedSegmentId = value;
    },

    // Tooltip state
    get tooltipData() {
      return tooltipData;
    },
    set tooltipData(value) {
      tooltipData = value;
    },

    // Computed tooltip (read-only)
    get displayedTooltip() {
      return displayedTooltip;
    },

    /**
     * Select a segment by ID. If the same segment is already selected, deselect it.
     * @param segmentId - The segment ID to select (or null to deselect)
     */
    selectSegment(segmentId: string | null) {
      if (selectedSegmentId === segmentId) {
        selectedSegmentId = null; // Toggle off
      } else {
        selectedSegmentId = segmentId;
      }
    },

    /**
     * Set hover state for a segment.
     * @param segmentId - The segment ID being hovered (or null when hover ends)
     */
    hoverSegment(segmentId: string | null) {
      hoveredSegmentId = segmentId;
    },

    /**
     * Update tooltip data for hover interactions.
     * @param data - Tooltip data or null to hide
     */
    updateTooltip(data: TooltipData | null) {
      tooltipData = data;
    },

    /**
     * Clear all interaction state (hover, selection, tooltip).
     * Useful when collapsing the card or resetting interactions.
     */
    clearAll() {
      hoveredFeature = undefined;
      hoveredSegmentId = null;
      selectedSegmentId = null;
      tooltipData = null;
    },
  };
}

/**
 * Type helper to extract the return type of createMapInteractionState.
 * Useful for component props that accept the state object.
 */
export type MapInteractionState = ReturnType<typeof createMapInteractionState>;
