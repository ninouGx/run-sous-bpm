# Run Sous BPM

An application that synchronizes Spotify listening history with Strava workout routes to visualize which songs were playing at different points during activities. The primary goal is to display music information overlaid on interactive maps showing Strava workout data.

## Architecture

- **Backend**: Rust with Axum web framework
- **Frontend**: SvelteKit with TypeScript
- **Database**: PostgreSQL with TimescaleDB for time-series data
- **Cache**: Moka (in-memory cache for OAuth state, CSRF tokens, PKCE verifiers)
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
- **Current Tables**: `users`, `oauth_tokens`, `activities`, `activity_streams`
- **Hypertables**: Ready for time-series optimization (activity_streams)
- **Users & OAuth tokens**: Spotify and Strava authentication (Strava complete)
- **Workout routes**: GPS data from Strava activities with full sync capability
- **Music timeline**: Spotify listening history with timestamps (planned)
- **Synchronized sessions**: Correlated music and workout data for map visualization (planned)

### API Integration

- **Strava** (Implemented):
  - OAuth2 with PKCE flow
  - Activity sync with automatic pagination
  - Stream data sync (GPS, heart rate, cadence, power, temperature)
  - Automatic token refresh on expiration
  - Endpoints: `/api/strava/activities/*`
- **Spotify** (Planned): Music listening history with timestamps
- **Data Synchronization** (Planned): Match music timestamps with GPS coordinates
- **Rate limiting**: Circuit breakers and backoff strategies
- **Token management**: Secure storage with refresh rotation

## Key Features

**Implemented:**
- [x] Strava OAuth integration with PKCE flow
- [x] Strava API client for activity and stream data
- [x] Activity sync endpoints with automatic token refresh
- [x] Database repositories for activities and time-series sensor data
- [x] Activity query endpoints with pagination support

**Planned:**
- [ ] Spotify OAuth integration for music history
- [ ] Interactive maps displaying workout routes (Leaflet/Mapbox)
- [ ] Music timeline overlay on GPS coordinates
- [ ] Synchronized playback visualization
- [ ] Real-time workout tracking with live music updates
- [ ] Statistical analysis of music tempo/energy vs performance
- [ ] Export capabilities for data analysis

## Security

- OAuth access/refresh tokens encrypted with AES-256 in PostgreSQL
- CSRF tokens and PKCE verifiers cached in-memory (Moka) with 10-minute TTL
- PKCE flow for all OAuth implementations
- State parameter validation prevents CSRF attacks
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

- **Current Tables**:
  - `users`: User accounts with password authentication
  - `oauth_tokens`: Encrypted OAuth tokens for Strava and Spotify
  - `activities`: Strava workout metadata (distance, duration, elevation, etc.)
  - `activity_streams`: Time-series sensor data (GPS, heart rate, cadence, power)
- **Hypertables**: Ready for conversion (activity_streams)
- **Continuous Aggregates**: Planned for real-time statistics computation
- **Retention Policies**: Planned 3-month rolling window for raw sensor data

## License

This project is licensed under the MIT License.

## Current Status

**Backend**: Strava integration fully implemented with OAuth, API client, database repositories, and REST endpoints. Ready for frontend integration and Spotify implementation.

**Frontend**: Initial SvelteKit setup with TailwindCSS and TanStack Query. UI components and routing in progress.

**Next Steps**:
1. Implement Spotify OAuth and API integration
2. Build frontend UI for activity visualization
3. Add map visualization with Leaflet/Mapbox
4. Implement music-workout synchronization logic