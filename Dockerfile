# Hermes Intelligence Pipeline - Production Dockerfile
# Task 43: Production Deployment Configuration
# Multi-stage build for optimized Indonesian intelligence pipeline deployment

# ============================================================================
# Stage 1: Builder - Compile Rust services with maximum optimization
# ============================================================================
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build all services with release profile optimization
# Use release profile for maximum performance in production
RUN cargo build --release --workspace

# ============================================================================
# Stage 2: Runtime - Minimal production image
# ============================================================================
FROM debian:bookworm-slim as runtime

# Install runtime dependencies for Indonesian intelligence pipeline
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create hermes user for security
RUN groupadd -r hermes && useradd -r -g hermes hermes

# Create application directories
RUN mkdir -p /app/bin /app/config /app/data /app/logs \
    && chown -R hermes:hermes /app

# Set working directory
WORKDIR /app

# Copy compiled binaries from builder stage
COPY --from=builder /app/target/release/hermes-* /app/bin/

# Copy configuration files
COPY config/ /app/config/

# Create health check script
RUN echo '#!/bin/bash\ncurl -f http://localhost:8888/health || exit 1' > /app/bin/health-check.sh \
    && chmod +x /app/bin/health-check.sh

# Switch to hermes user
USER hermes

# Expose ports for all services
# 8881: hermes-collector
# 8882: hermes-processor  
# 8883: hermes-social
# 8884: hermes-economic
# 8885: hermes-analyst
# 8890: health-check endpoints
EXPOSE 8881 8882 8883 8884 8885 8890

# Health check for Indonesian intelligence pipeline
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD /app/bin/health-check.sh

# Default command runs the main intelligence backend
CMD ["/app/bin/hermes-analyst"]

# ============================================================================
# Alternative service-specific targets
# ============================================================================

# Collector service
FROM runtime as collector
CMD ["/app/bin/hermes-collector"]

# Processor service  
FROM runtime as processor
CMD ["/app/bin/hermes-processor"]

# Social intelligence service
FROM runtime as social
CMD ["/app/bin/hermes-social"]

# Economic intelligence service
FROM runtime as economic
CMD ["/app/bin/hermes-economic"]

# Analyst service (Prof Jiang framework)
FROM runtime as analyst
CMD ["/app/bin/hermes-analyst"]

# ============================================================================
# Multi-architecture build support
# ============================================================================
# Build for both AMD64 (Intel/AMD) and ARM64 (Apple Silicon, ARM servers)
# Usage: docker buildx build --platform linux/amd64,linux/arm64 -t hermes-intelligence .

# ============================================================================
# Production Environment Variables
# ============================================================================
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    HERMES_ENV=production \
    ARANGO_URL=arangodb://arangodb:8529 \
    ARANGO_DATABASE=hermes \
    ARANGO_USERNAME=hermes \
    # Indonesian market configuration
    INDONESIAN_STOCKS="BMRI,BBRI,INCO,ANTM,PTBA,TAPG" \
    PROF_JIANG_ENABLED=true \
    GEOPOLITICAL_ANALYSIS=true \
    # Security configuration
    JWT_SECRET_FILE=/run/secrets/jwt_secret \
    RATE_LIMIT_RPM=300 \
    # Performance tuning
    TOKIO_WORKER_THREADS=4 \
    MAX_CONNECTIONS=100

# ============================================================================
# Build instructions for production deployment:
#
# 1. Build all services:
#    docker build -t hermes-intelligence:latest .
#
# 2. Build specific service:
#    docker build --target collector -t hermes-collector:latest .
#
# 3. Multi-architecture build:
#    docker buildx build --platform linux/amd64,linux/arm64 -t hermes-intelligence:latest .
#
# 4. Production deployment with compose:
#    docker-compose -f docker-compose.prod.yml up -d
#
# ============================================================================