import { type } from "arktype";

export const stravaActivitySchema = type({
  id: "number",
  name: "string",
  type: "string",
  distance: "number>=0",
  moving_time: "number>=0",
  elapsed_time: "number>=0",
  total_elevation_gain: "number>=0",
  start_date: "string",
  timezone: "string",
  "description?": "string",
});

export const activitySyncSchema = type({
  activity_id: "number",
  sync_music: "boolean",
  "date_range?": {
    start: "string",
    end: "string",
  },
});

export type StravaActivityData = typeof stravaActivitySchema.infer;
export type ActivitySyncData = typeof activitySyncSchema.infer;
