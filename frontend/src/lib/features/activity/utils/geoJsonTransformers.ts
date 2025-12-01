/**
 * GeoJSON transformation utilities for converting activity data to map-ready formats.
 */

import type { MusicSegment, ActivityStream } from "$lib/shared/api/types";
import { getTrackColor } from "./track-colors";

/**
 * Represents bounding box coordinates for map viewport.
 */
export interface Bounds {
  minLng: number;
  maxLng: number;
  minLat: number;
  maxLat: number;
}

/**
 * Creates a GeoJSON FeatureCollection from music segments.
 * Each segment becomes a LineString feature with track metadata.
 *
 * @param segments - Array of music segments with GPS points
 * @returns GeoJSON FeatureCollection with LineString features
 *
 * @example
 * ```typescript
 * const geoJSON = createRouteGeoJSON(segments);
 * // Use with MapLibre: <GeoJSONSource data={geoJSON} />
 * ```
 */
export function createRouteGeoJSON(
  segments: MusicSegment[],
): GeoJSON.FeatureCollection {
  return {
    type: "FeatureCollection",
    features: segments.map((segment) => ({
      type: "Feature",
      geometry: {
        type: "LineString",
        coordinates: segment.points
          .filter(
            (point) =>
              typeof point.latitude === "number" &&
              typeof point.longitude === "number",
          )
          .map((point) => [point.longitude, point.latitude]),
      },
      properties: {
        segment_id: segment.index,
        track_name: segment.track?.track_name || null,
        artist_name: segment.track?.artist_name || null,
        color: getTrackColor(segment.index),
      },
    })),
  };
}

/**
 * Calculate bounding box from activity stream GPS coordinates.
 * Returns null if no valid coordinates exist.
 *
 * @param activityStream - Array of GPS points from activity
 * @returns Bounding box or null if empty/invalid
 *
 * @example
 * ```typescript
 * const bounds = calculateBounds(activityStream);
 * if (bounds) {
 *   map.fitBounds([
 *     [bounds.minLng, bounds.minLat],
 *     [bounds.maxLng, bounds.maxLat]
 *   ]);
 * }
 * ```
 */
export function calculateBounds(activityStream: ActivityStream): Bounds | null {
  if (activityStream.length === 0) return null;

  const lngs = activityStream.map((p) => p.longitude);
  const lats = activityStream.map((p) => p.latitude);

  return {
    minLng: Math.min(...lngs),
    maxLng: Math.max(...lngs),
    minLat: Math.min(...lats),
    maxLat: Math.max(...lats),
  };
}
