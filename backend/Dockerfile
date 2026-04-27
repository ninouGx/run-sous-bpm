FROM lukemathwalker/cargo-chef:latest-rust-1.92-slim-bookworm AS chef
WORKDIR /app

# --- Stage: planner (captures dependency graph) ---
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- Stage: builder (builds deps then app) ---
FROM chef AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin run-sous-bpm-api --bin migration

# --- Stage: runtime ---
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/run-sous-bpm-api .
COPY --from=builder /app/target/release/migration .

EXPOSE 3000
CMD ["./run-sous-bpm-api"]
