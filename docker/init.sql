-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Create basic tables for initial setup
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create hypertable for workout metrics (time-series data)
CREATE TABLE IF NOT EXISTS workout_metrics (
    time TIMESTAMPTZ NOT NULL,
    user_id INTEGER REFERENCES users(id),
    workout_id VARCHAR(255) NOT NULL,
    metric_type VARCHAR(100) NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    unit VARCHAR(50)
);

-- Convert to hypertable
SELECT create_hypertable('workout_metrics', 'time', if_not_exists => TRUE);

-- Create hypertable for music events (time-series data)
CREATE TABLE IF NOT EXISTS music_events (
    time TIMESTAMPTZ NOT NULL,
    user_id INTEGER REFERENCES users(id),
    track_id VARCHAR(255) NOT NULL,
    artist VARCHAR(255),
    track_name VARCHAR(255),
    event_type VARCHAR(50) NOT NULL -- 'play', 'pause', 'skip', etc.
);

-- Convert to hypertable
SELECT create_hypertable('music_events', 'time', if_not_exists => TRUE);