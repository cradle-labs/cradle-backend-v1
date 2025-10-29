# Use official Rust image as builder
FROM rust:1.90.0-bookworm as chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json

# Install system dependencies needed for compilation
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies - this layer will be cached unless dependencies change
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code and build the application
COPY . .
RUN cargo build --release

# Runtime stage - minimal image
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -r -s /bin/false appuser

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/cradle-back-end /app/cradle-back-end

# Change ownership to app user
RUN chown appuser:appuser /app/cradle-back-end

# Switch to non-root user
USER appuser

# Expose port (adjust if your app uses a different port)
EXPOSE 6969

# Run the binary
CMD ["./cradle-back-end"]
