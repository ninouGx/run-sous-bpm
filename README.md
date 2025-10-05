# Run Sous BPM

An application that synchronizes Spotify listening history with Strava workout routes to visualize which songs were playing at different points during activities. The primary goal is to display music information overlaid on interactive maps showing Strava workout data.

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
├── backend/                      # Rust workspace
│   ├── Cargo.toml               # Workspace definition with shared dependencies
│   ├── api/                     # Axum web server & REST endpoints
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── core/                    # Business logic & domain models
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── models/         # Domain models
│   │   │   ├── services/       # Business logic
│   │   │   └── database/       # Database module
│   │   │       ├── mod.rs      # Connection utilities
│   │   │       ├── connection.rs
│   │   │       └── entities/   # SeaORM generated entities
│   │   └── Cargo.toml
│   ├── integrations/            # External API clients (Strava, Spotify)
│   │   ├── src/
│   │   └── Cargo.toml
│   └── migration/               # SeaORM database migrations
│       ├── src/
│       │   ├── lib.rs
│       │   ├── main.rs         # Migration CLI
│       │   └── m*.rs           # Migration files
│       └── Cargo.toml
├── frontend/                    # SvelteKit application
│   ├── src/
│   │   ├── routes/             # SvelteKit routes
│   │   └── lib/                # Shared components
│   └── package.json
└── docker-compose.yml          # PostgreSQL + TimescaleDB
```

## Development Workflow

### Database Management

**Run migrations:**
```bash
cd backend
cargo run --package migration
```

**Generate entities from schema:**
```bash
cd backend
sea-orm-cli generate entity -o core/src/database/entities
```

The TimescaleDB database includes:
- **Hypertables**: Optimized for GPS coordinates and music events
- **Users & OAuth tokens**: Spotify and Strava authentication
- **Workout routes**: GPS data from Strava activities
- **Music timeline**: Spotify listening history with timestamps
- **Synchronized sessions**: Correlated music and workout data for map visualization

### API Integration

- **Strava**: Workout GPS routes and activity data
- **Spotify**: Music listening history with timestamps
- **Data Synchronization**: Match music timestamps with GPS coordinates
- **Rate limiting**: Circuit breakers and backoff strategies
- **Token management**: Secure storage with refresh rotation

## Key Features (Planned)

- [ ] Spotify OAuth integration for music history
- [ ] Strava OAuth integration for workout data
- [ ] Interactive maps displaying workout routes
- [ ] Music timeline overlay on GPS coordinates
- [ ] Synchronized playback visualization
- [ ] Future: Real-time workout tracking with live music updates
- [ ] Future: Statistical analysis of music tempo/energy vs performance
- [ ] Future: Export capabilities for data analysis

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