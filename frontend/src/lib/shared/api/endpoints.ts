import type { OauthProvider } from "$lib/shared/api/types";

export const API_ENDPOINTS = {
  auth: {
    register: "/api/auth/register",
    login: "/api/auth/login",
    logout: "/api/auth/logout",
    me: "/api/auth/me",
  },
  oauth: {
    authorize: (provider: OauthProvider) => `/api/oauth/${provider}/authorize`,
    disconnect: (provider: OauthProvider) =>
      `/api/oauth/${provider}/disconnect`,
  },
  user: {
    update: "/api/user",
  },
  strava: {
    activities: "/api/strava/activities",
    syncActivities: "/api/strava/activities/sync",
    activityStreams: (id: string) => `/api/strava/activities/${id}/streams`,
    syncActivityStreams: (id: string) =>
      `/api/strava/activities/${id}/streams/sync`,
    syncAllActivityStreams: "/api/strava/activities/streams/sync",
  },
  activities: {
    music: (activityId: string) => `/api/activities/${activityId}/music`,
  },
} as const;
