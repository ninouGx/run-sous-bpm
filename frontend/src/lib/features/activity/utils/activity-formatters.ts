import type { StravaActivity } from "$lib/shared/api/types";

/**
 * Format distance from meters to km with 2 decimal places
 */
export function formatDistance(meters: number): string {
  const km = meters / 1000;
  return `${km.toFixed(2)} km`;
}

/**
 * Format time from seconds to human readable format
 * Examples: "1h 23m", "45m 30s", "12s"
 */
export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
}

/**
 * Calculate and format pace (min/km)
 */
export function formatPace(
  distanceMeters: number,
  timeSeconds: number,
): string {
  if (distanceMeters === 0) return "—";

  const km = distanceMeters / 1000;
  const minutes = timeSeconds / 60;
  const paceMinPerKm = minutes / km;

  const mins = Math.floor(paceMinPerKm);
  const secs = Math.round((paceMinPerKm - mins) * 60);

  return `${mins}:${secs.toString().padStart(2, "0")} /km`;
}

/**
 * Calculate and format speed (km/h)
 */
export function formatSpeed(
  distanceMeters: number,
  timeSeconds: number,
): string {
  if (timeSeconds === 0) return "—";

  const km = distanceMeters / 1000;
  const hours = timeSeconds / 3600;
  const speed = km / hours;

  return `${speed.toFixed(1)} km/h`;
}

/**
 * Format elevation gain
 */
export function formatElevation(meters: number): string {
  return `${Math.round(meters)} m`;
}

/**
 * Format activity date
 */
export function formatActivityDate(
  dateString: string,
  is24Hour: boolean = true,
): string {
  const date = new Date(dateString);
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: !is24Hour,
  }).format(date);
}

/**
 * Format time to HH:MM depending on wanted format 24h or 12h
 */
export function formatTime(
  dateString: string,
  is24Hour: boolean = true,
): string {
  const date = new Date(dateString);
  return new Intl.DateTimeFormat("en-US", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: !is24Hour,
  }).format(date);
}
/**
 * Get a relative time string (e.g., "2 hours ago", "3 days ago")
 */
export function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffDays > 30) {
    return formatActivityDate(dateString);
  } else if (diffDays > 0) {
    return `${diffDays} day${diffDays > 1 ? "s" : ""} ago`;
  } else if (diffHours > 0) {
    return `${diffHours} hour${diffHours > 1 ? "s" : ""} ago`;
  } else if (diffMins > 0) {
    return `${diffMins} minute${diffMins > 1 ? "s" : ""} ago`;
  } else {
    return "Just now";
  }
}

/**
 * Get activity type emoji
 */
export function getActivityEmoji(type: string): string {
  const emojiMap: Record<string, string> = {
    Run: "🏃",
    Ride: "🚴",
    Walk: "🚶",
    Hike: "🥾",
    Swim: "🏊",
    Workout: "💪",
    Yoga: "🧘",
    default: "🏃",
  };

  return emojiMap[type] || emojiMap.default;
}

/**
 * Calculate activity stats summary
 */
export function calculateActivityStats(activities: StravaActivity[]) {
  const totalDistance = activities.reduce((sum, a) => sum + a.distance, 0);
  const totalTime = activities.reduce((sum, a) => sum + a.moving_time, 0);
  const totalElevation = activities.reduce(
    (sum, a) => sum + a.total_elevation_gain,
    0,
  );

  return {
    totalDistance: formatDistance(totalDistance),
    totalTime: formatDuration(totalTime),
    totalElevation: formatElevation(totalElevation),
    count: activities.length,
    avgDistance:
      activities.length > 0
        ? formatDistance(totalDistance / activities.length)
        : "—",
    avgSpeed:
      activities.length > 0 ? formatSpeed(totalDistance, totalTime) : "—",
  };
}
