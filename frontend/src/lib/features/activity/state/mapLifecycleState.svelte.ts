/**
 * State management for MapLibre lazy loading and lifecycle.
 * Handles component loading, map instance management, and cleanup.
 */

import type { Map } from "maplibre-gl";

/**
 * Creates a reactive state object for managing MapLibre lifecycle.
 * Handles lazy loading of map components and proper cleanup on unmount.
 *
 * @returns State object with map instance and loading flags
 *
 * @example
 * ```svelte
 * <script>
 *   const lifecycle = createMapLifecycleState();
 *
 *   // In $effect - lazy load components when expanded
 *   $effect(() => {
 *     if (isExpanded && !lifecycle.componentsLoaded) {
 *       lifecycle.loadComponents();
 *     }
 *   });
 *
 *   // In $effect - cleanup when collapsed
 *   $effect(() => {
 *     if (!isExpanded && lifecycle.map) {
 *       lifecycle.cleanup();
 *     }
 *   });
 * </script>
 * ```
 */
export function createMapLifecycleState() {
  // Lazy-loaded MapLibre components
  let MapLibre = $state<any>(null);
  let NavigationControl = $state<any>(null);
  let ScaleControl = $state<any>(null);
  let GeoJSONSource = $state<any>(null);
  let LineLayer = $state<any>(null);
  let FeatureState = $state<any>(null);
  let componentsLoaded = $state(false);

  // Map instance and style loading
  let map = $state<Map | undefined>(undefined);
  let isMapStyleLoaded = $state(false);

  return {
    // Lazy-loaded components
    get MapLibre() {
      return MapLibre;
    },
    get NavigationControl() {
      return NavigationControl;
    },
    get ScaleControl() {
      return ScaleControl;
    },
    get GeoJSONSource() {
      return GeoJSONSource;
    },
    get LineLayer() {
      return LineLayer;
    },
    get FeatureState() {
      return FeatureState;
    },
    get componentsLoaded() {
      return componentsLoaded;
    },

    // Map instance
    get map() {
      return map;
    },
    set map(value) {
      map = value;
    },

    // Style loading flag
    get isMapStyleLoaded() {
      return isMapStyleLoaded;
    },
    set isMapStyleLoaded(value) {
      isMapStyleLoaded = value;
    },

    /**
     * Lazy load MapLibre components.
     * Only loads once, subsequent calls are no-ops.
     *
     * @returns Promise that resolves when components are loaded
     */
    async loadComponents() {
      if (componentsLoaded) return;

      const module = await import("svelte-maplibre-gl");
      MapLibre = module.MapLibre;
      NavigationControl = module.NavigationControl;
      ScaleControl = module.ScaleControl;
      GeoJSONSource = module.GeoJSONSource;
      LineLayer = module.LineLayer;
      FeatureState = module.FeatureState;
      componentsLoaded = true;
    },

    /**
     * Clean up map instance and reset state.
     * Safe to call multiple times - handles already-removed maps gracefully.
     */
    cleanup() {
      if (map) {
        try {
          map.remove();
        } catch (e) {
          // Map already removed, ignore
        }
        map = undefined;
      }
      isMapStyleLoaded = false;
      componentsLoaded = false;
    },

    /**
     * Reset all state to initial values.
     * More aggressive than cleanup - also clears component references.
     */
    reset() {
      this.cleanup();
      MapLibre = null;
      NavigationControl = null;
      ScaleControl = null;
      GeoJSONSource = null;
      LineLayer = null;
      FeatureState = null;
    },
  };
}

/**
 * Type helper to extract the return type of createMapLifecycleState.
 */
export type MapLifecycleState = ReturnType<typeof createMapLifecycleState>;
