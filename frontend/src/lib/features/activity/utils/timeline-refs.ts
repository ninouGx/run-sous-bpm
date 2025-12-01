/**
 * Svelte action for managing timeline item references with automatic cleanup.
 * Used to track DOM elements for scroll synchronization with map interactions.
 */

import type { Action } from "svelte/action";

/**
 * Creates a ref management system for timeline items.
 * Returns an action that can be used with Svelte's `use:` directive.
 *
 * @returns Object with action and ref map
 *
 * @example
 * ```svelte
 * <script>
 *   const { bindTimelineItemRef, timelineItemRefs } = createTimelineRefs();
 *
 *   // Scroll to a specific item
 *   const element = timelineItemRefs.get('segment-1');
 *   if (element) {
 *     element.scrollIntoView({ behavior: 'smooth' });
 *   }
 * </script>
 *
 * <div use:bindTimelineItemRef={'segment-1'}>
 *   Track item content
 * </div>
 * ```
 */
export function createTimelineRefs() {
  // Using built-in Map, not maplibre Map
  const timelineItemRefs = new Map<string, HTMLElement>();

  /**
   * Svelte action to bind an element to the ref map.
   * Automatically cleans up on element destruction.
   */
  const bindTimelineItemRef: Action<HTMLElement, string> = (
    node: HTMLElement,
    segmentId: string,
  ) => {
    timelineItemRefs.set(segmentId, node);

    return {
      destroy() {
        timelineItemRefs.delete(segmentId);
      },
    };
  };

  return {
    bindTimelineItemRef,
    timelineItemRefs,
  };
}

/**
 * Scrolls a timeline item into view with smooth animation.
 *
 * @param timelineItemRefs - Map of segment IDs to HTML elements
 * @param segmentId - The segment ID to scroll to
 *
 * @example
 * ```typescript
 * const { timelineItemRefs } = createTimelineRefs();
 * scrollToTimelineItem(timelineItemRefs, 'segment-1');
 * ```
 */
export function scrollToTimelineItem(
  timelineItemRefs: Map<string, HTMLElement>,
  segmentId: string | null,
): void {
  if (segmentId === null) return;

  const element = timelineItemRefs.get(segmentId);
  if (element) {
    element.scrollIntoView({
      behavior: "smooth",
      block: "nearest",
      inline: "nearest",
    });
  }
}
