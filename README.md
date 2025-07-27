# Run Sous BPM

A fitness-music analytics platform that correlates fitness activities (Strava) with music listening data (Spotify/Last.fm) to be displayed over maps and .

## Architecture

- **Backend**: Rust with Axum web framework
- **Frontend**: SvelteKit with TypeScript
- **Database**: PostgreSQL with TimescaleDB for time-series data
- **Cache**: Redis
- **Authentication**: Multi-provider OAuth2 with PKCE

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [Docker](https://www.docker.com/) and Docker Compose

### 1. Clone and Setup

```bash
git clone <repository-url>
cd run-sous-bpm
```

### 2. Start Database Services

```bash
docker-compose up -d
```

This starts:
- PostgreSQL with TimescaleDB (port 5433)
- Redis (port 6379)

### 3. Backend Development

```bash
cd backend
cargo run
```

Available commands:
```bash
cargo watch -x run    # Hot reload development
cargo build --release # Production build
cargo test            # Run tests
cargo clippy          # Lint code
cargo fmt             # Format code
```

### 4. Frontend Development

```bash
cd frontend
npm install
npm run dev
```

Available commands:
```bash
npm run dev      # Development server
npm run build    # Production build
npm run preview  # Preview production build
npm run check    # Type checking
npm run lint     # Lint and format
```

## Project Structure

```
run-sous-bpm/
├── backend/                 # Rust workspace
│   ├── api/                # Axum web server
│   ├── core/               # Business logic
│   └── integrations/       # External API clients
├── frontend/               # SvelteKit application
│   ├── src/
│   │   ├── routes/        # SvelteKit routes
│   │   └── lib/           # Shared components
└── docker/                # Database initialization
```

## Development Workflow

### Database Management

The TimescaleDB database includes:
- **Hypertables**: Optimized for time-series data
- **Users & OAuth tokens**: Multi-provider authentication
- **Workout metrics**: Activity data from Strava
- **Music events**: Listening data correlated with workouts

### API Integration

- **Strava**: Workout data and real-time webhooks
- **Spotify/Last.fm**: Music listening history
- **Rate limiting**: Circuit breakers and backoff strategies
- **Token management**: Secure storage with refresh rotation

## Key Features (Planned)

- [ ] Multi-provider OAuth authentication
- [ ] Real-time workout tracking
- [ ] Music correlation analytics
- [ ] Interactive data visualizations
- [ ] Performance insights dashboard
- [ ] Export capabilities

## Security

- OAuth tokens encrypted with AES-256
- PKCE flow for all OAuth implementations
- Rate limiting: 1000 requests/hour per user
- GDPR compliance with data retention policies
- Input validation on all API endpoints

## Performance Targets

- API response time: < 100ms (p95)
- Dashboard load time: < 1s
- Real-time updates: < 500ms latency
- Support for 1000+ concurrent users

## Testing

### Backend Tests
```bash
cd backend && cargo test
```

### Frontend Tests
```bash
cd frontend && npm test
```

## Database Schema

The project uses PostgreSQL with TimescaleDB extension:

- **Hypertables**: `workout_metrics` and `music_events` for sensor data
- **Continuous Aggregates**: Real-time statistics computation
- **Retention Policies**: 3-month rolling window for raw sensor data

## License

This project is licensed under the MIT License.

## Current Status

This repository contains the initial project setup and architectural foundation. The core features are currently in development following the documented structure.