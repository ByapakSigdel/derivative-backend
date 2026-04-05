# Build stage
FROM rust:1.75-bookworm as builder
WORKDIR /app

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml ./
# Create Cargo.lock if it doesn't exist
RUN cargo generate-lockfile

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release
RUN rm src/main.rs

# Copy actual source code
COPY src ./src

# Build the actual application
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

# Copy binary from builder
COPY --from=builder /app/target/release/derivative-backend /usr/local/bin/derivative-backend

# Create upload directories
RUN mkdir -p /app/uploads/avatars /app/uploads/project_assets \
    && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the binary
CMD ["derivative-backend"]