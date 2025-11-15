import { apiClient } from "$lib/shared/api/client";
import { API_ENDPOINTS } from "$lib/shared/api/endpoints";
import { userStore } from "$lib/stores/user";
import type {
  LoginRequest,
  RegisterRequest,
  User,
  AuthResponse,
  OauthProvider,
} from "$lib/shared/api/types";

export const oauthService = {
  async getAuthorizationUrl(provider: OauthProvider): Promise<string> {
    let endpoint: string;
    endpoint = API_ENDPOINTS.oauth.authorize(provider);
    const response = await apiClient.get<{ auth_url: string }>(endpoint);
    return response.auth_url;
  },

  async disconnectProvider(provider: OauthProvider): Promise<void> {
    let endpoint: string;
    endpoint = API_ENDPOINTS.oauth.disconnect(provider);
    await apiClient.post<{ message: string }>(endpoint);
    userStore.updateOauthConnection(provider, false);
    return;
  },
};
