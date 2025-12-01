/**
 * Track color utilities for music visualization on activity maps.
 * Provides a consistent color palette for displaying music tracks along GPS routes.
 */

/**
 * Predefined color palette for track visualization.
 * Colors chosen for high visibility on map backgrounds.
 */
export const TRACK_COLORS = [
  "#60A5FA", // Bleu ciel vif (très lumineux)
  "#818CF8", // Indigo moyen
  "#A855F7", // Violet électrique (saturé)
  "#3B82F6", // Bleu roi (foncé mais vif)
  "#C084FC", // Lavande lumineuse
] as const;

/**
 * Get a color for a track segment by index.
 * Colors cycle through the palette using modulo.
 *
 * @param index - The segment index (0-based)
 * @returns Hex color string (e.g., "#60A5FA")
 *
 * @example
 * ```typescript
 * const color = getTrackColor(0); // "#60A5FA"
 * const color2 = getTrackColor(5); // "#60A5FA" (cycles back to first color)
 * ```
 */
export function getTrackColor(index: number): string {
  return TRACK_COLORS[index % TRACK_COLORS.length];
}

/**
 * Get a color with alpha transparency for a track segment.
 * Useful for hover/selection effects or layering.
 *
 * @param index - The segment index (0-based)
 * @param alpha - Alpha value (0.0 to 1.0)
 * @returns Color string with alpha channel
 *
 * @example
 * ```typescript
 * const color = getTrackColorWithAlpha(0, 0.5); // "#60A5FA / 0.5"
 * ```
 */
export function getTrackColorWithAlpha(index: number, alpha: number): string {
  const color = TRACK_COLORS[index % TRACK_COLORS.length];
  return color.replace(")", ` / ${alpha})`);
}
