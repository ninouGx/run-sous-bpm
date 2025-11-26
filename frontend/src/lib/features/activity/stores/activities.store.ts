import { writable, derived } from "svelte/store";
import type { StravaActivity } from "$lib/shared/api/types";
import { activitiesService } from "../activity.service";

interface ActivitiesState {
  activities: StravaActivity[];
  isLoading: boolean;
  error: string | null;
  lastSync: Date | null;
}

function createActivitiesStore() {
  const { subscribe, set, update } = writable<ActivitiesState>({
    activities: [],
    isLoading: false,
    error: null,
    lastSync: null,
  });

  return {
    subscribe,

    /**
     * Load activities from the API
     */
    async load() {
      update((state) => ({ ...state, isLoading: true, error: null }));

      try {
        const activities = await activitiesService.getActivities();
        update((state) => ({
          ...state,
          activities,
          isLoading: false,
          error: null,
        }));
      } catch (error: unknown) {
        update((state) => ({
          ...state,
          isLoading: false,
          error: error instanceof Error ? error.message : "Failed to load activities",
        }));
      }
    },

    /**
     * Sync activities from Strava
     */
    async sync() {
      update((state) => ({ ...state, isLoading: true, error: null }));

      try {
        await activitiesService.syncActivities();
        await activitiesService.syncAllActivityStreams();
        const activities = await activitiesService.getActivities();
        update((state) => ({
          ...state,
          activities,
          isLoading: false,
          error: null,
          lastSync: new Date(),
        }));
      } catch (error: unknown) {
        update((state) => ({
          ...state,
          isLoading: false,
          error: error instanceof Error ? error.message : "Failed to sync activities",
        }));
      }
    },

    /**
     * Clear all activities
     */
    clear() {
      set({
        activities: [],
        isLoading: false,
        error: null,
        lastSync: null,
      });
    },

    /**
     * Reset error state
     */
    clearError() {
      update((state) => ({ ...state, error: null }));
    },
  };
}

export const activitiesStore = createActivitiesStore();

// Derived stores for common queries
export const sortedActivities = derived(activitiesStore, ($store) =>
  [...$store.activities].sort(
    (a, b) => new Date(b.start_time).getTime() - new Date(a.start_time).getTime()
  )
);

export const activityTypes = derived(activitiesStore, ($store) => {
  const types = new Set($store.activities.map((a) => a.type));
  return Array.from(types).sort();
});
