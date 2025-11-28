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
  id: string; // Internal UUID
  name: string;
  type: string;
  distance: number;
  moving_time: number;
  elapsed_time: number;
  total_elevation_gain: number;
  start_time: string;
  timezone: string;
  description?: string;
}

// Music types
export interface TrackInfo {
  id: string;
  track_name: string;
  artist_name: string;
  album_name?: string;
}

// TODO: Remove this compatibility type once UI is updated to use segments directly
export interface TrackWithTimestamp {
  played_at: string;
  track_name: string;
  artist_name: string;
  album_name?: string;
  track_id: string;
  listen_id: string;
}

export interface GpsPoint {
  time: string;
  latitude: number;
  longitude: number;
  altitude?: number;
  heart_rate?: number;
  cadence?: number;
  watts?: number;
  velocity?: number;
}

export interface MusicSegment {
  index: number;
  track?: TrackInfo;
  start_time: string;
  end_time: string;
  points: GpsPoint[];
}

export interface SimplificationStats {
  total_segments: number;
  segments_with_music: number;
  segments_without_music: number;
  original_points: number;
  simplified_points: number;
  reduction_ratio: number;
}

export interface ActivityMusicResponse {
  activity_id: string;
  has_gps: boolean;
  segments: MusicSegment[];
  stats: SimplificationStats;
}

export interface ActivityStreamPoint {
  activity_id: string;
  time: string;
  distance: number;
  latitude: number;
  longitude: number;
  altitude: number;
  velocity: number;
  heart_rate: number | null;
  cadence: number | null;
  watts: number | null;
  temperature: number | null;
}

export type ActivityStream = ActivityStreamPoint[];
