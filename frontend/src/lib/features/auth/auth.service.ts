import { apiClient } from "$lib/shared/api/client";
import { API_ENDPOINTS } from "$lib/shared/api/endpoints";
import { userStore } from "$lib/stores/user";
import type {
  LoginRequest,
  RegisterRequest,
  User,
  AuthResponse,
} from "$lib/shared/api/types";

export const authService = {
  async checkAuth(): Promise<User | null> {
    try {
      userStore.setLoading(true);
      const user = await apiClient.get<User>(API_ENDPOINTS.auth.me);
      userStore.setUser(user);
      return user;
    } catch (error) {
      userStore.clearUser();
      return null;
    }
  },

  async login(credentials: LoginRequest): Promise<User> {
    const response = await apiClient.post<AuthResponse>(
      API_ENDPOINTS.auth.login,
      credentials,
    );
    userStore.setUser(response.user);
    return response.user;
  },

  async register(credentials: RegisterRequest): Promise<User> {
    const response = await apiClient.post<{ id: string; email: string }>(
      API_ENDPOINTS.auth.register,
      credentials,
    );
    return response;
  },

  async logout(): Promise<void> {
    await apiClient.post(API_ENDPOINTS.auth.logout);
    userStore.clearUser();
  },
};
