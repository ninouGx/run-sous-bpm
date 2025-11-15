// API Error types
export interface ApiError {
  error: string;
  message?: string;
  status: number;
}

// Auth types
export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
}

export enum OauthProvider {
  Strava = "strava",
  Spotify = "spotify",
}

export type OauthConnection = {
  [provider in OauthProvider]: boolean;
};

export interface User {
  id: string;
  email: string;
  lastfm_username?: string | null;
  oauth_connections?: OauthConnection;
}

export interface AuthResponse {
  message: string;
  user: User;
}

// Strava types
export interface StravaActivity {
  id: number;
  name: string;
  type: string;
  distance: number;
  moving_time: number;
  elapsed_time: number;
  total_elevation_gain: number;
  start_date: string;
  timezone: string;
  description?: string;
}

// Music types
export interface TrackWithTimestamp {
  played_at: string;
  track_name: string;
  artist_name: string;
  album_name?: string;
  track_id: string;
  listen_id: string;
}

export interface ActivityMusicResponse {
  tracks: TrackWithTimestamp[];
  total_tracks: number;
}
