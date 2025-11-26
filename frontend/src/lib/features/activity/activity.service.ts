import { apiClient } from "$lib/shared/api/client";
import { API_ENDPOINTS } from "$lib/shared/api/endpoints";
import type { ActivityStream, StravaActivity } from "$lib/shared/api/types";

class ActivitiesService {
  /**
   * Fetch all Strava activities for the authenticated user
   */
  async getActivities(): Promise<StravaActivity[]> {
    const response = await apiClient.get<StravaActivity[]>(
      API_ENDPOINTS.strava.activities
    );
    return response;
  }

  /**
   * Sync activities from Strava
   * This triggers a fetch from Strava API and updates the database
   */
  async syncActivities(): Promise<{ message: string; count: number }> {
    const response = await apiClient.post<{ message: string; count: number }>(
      API_ENDPOINTS.strava.syncActivities
    );
    return response;
  }

  /**
   * Get a specific activity by ID
   */
  async getActivity(id: string): Promise<StravaActivity> {
    const activities = await this.getActivities();
    const activity = activities.find((a) => a.id.toString() === id);
    
    if (!activity) {
      throw new Error(`Activity ${id} not found`);
    }
    
    return activity;
  }

  /**
   * Get activity streams (heart rate, cadence, etc.)
   * @param id - The internal UUID of the activity
   */
  async getActivityStreams(id: string): Promise<ActivityStream> {
    const response = await apiClient.get<ActivityStream>(
      API_ENDPOINTS.strava.activityStreams(id)
    );
    return response;
  }

  /**
   * Sync activity streams from Strava
   * @param id - The internal UUID of the activity
   */
  async syncActivityStreams(id: string): Promise<{ message: string }> {
    const response = await apiClient.post<{ message: string }>(
      API_ENDPOINTS.strava.syncActivityStreams(id)
    );
    return response;
  }

  /**
   * Sync all activity streams from Strava
   */
  async syncAllActivityStreams(): Promise<{ message: string }> {
    const response = await apiClient.post<{ message: string }>(
      API_ENDPOINTS.strava.syncAllActivityStreams
    );
    return response;
  }
}

export const activitiesService = new ActivitiesService();
