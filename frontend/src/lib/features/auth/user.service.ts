import { apiClient } from "$lib/shared/api/client";
import { API_ENDPOINTS } from "$lib/shared/api/endpoints";
import { userStore } from "$lib/stores/user";

export interface UpdateLastfmUsernameRequest {
  lastfm_username: string;
}

export interface UpdateLastfmUsernameResponse {
  message: string;
}

export const userService = {
  async updateLastfmUsername(username: string): Promise<void> {
    const response = await apiClient.patch<UpdateLastfmUsernameResponse>(
      API_ENDPOINTS.user.update,
      { lastfm_username: username } satisfies UpdateLastfmUsernameRequest,
    );

    // Update the user store with the new lastfm_username
    userStore.updateLastfmUsername(username);

    return;
  },
};
