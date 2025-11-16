import { apiClient } from "$lib/shared/api/client";
import { API_ENDPOINTS } from "$lib/shared/api/endpoints";
import type { ActivityMusicResponse } from "$lib/shared/api/types";

class ActivityMusicService {
  /**
   * Get music tracks played during an activity
   */
  async getActivityMusic(activityId: string): Promise<ActivityMusicResponse> {
    const response = await apiClient.get<ActivityMusicResponse>(
      API_ENDPOINTS.activities.music(activityId)
    );
    return response;
  }
}

export const activityMusicService = new ActivityMusicService();
