import { writable } from "svelte/store";
import type { User } from "$lib/shared/api/types";

interface UserState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

function createUserStore() {
  const { subscribe, set, update } = writable<UserState>({
    user: null,
    isAuthenticated: false,
    isLoading: true,
  });

  return {
    subscribe,
    setUser: (user: User) =>
      update((state) => ({
        ...state,
        user,
        isAuthenticated: true,
        isLoading: false,
      })),
    clearUser: () =>
      set({
        user: null,
        isAuthenticated: false,
        isLoading: false,
      }),
    setLoading: (isLoading: boolean) =>
      update((state) => ({
        ...state,
        isLoading,
      })),
    updateLastfmUsername: (lastfmUsername: string) =>
      update((state) => ({
        ...state,
        user: state.user
          ? { ...state.user, lastfm_username: lastfmUsername }
          : null,
      })),
    updateOauthConnection: (provider: string, isConnected: boolean) =>
      update((state) => {
        if (!state.user) return state;

        return {
          ...state,
          user: {
            ...state.user,
            oauth_connections: {
              ...state.user.oauth_connections,
              [provider]: isConnected,
            } as User["oauth_connections"],
          },
        };
      }),
  };
}

export const userStore = createUserStore();
