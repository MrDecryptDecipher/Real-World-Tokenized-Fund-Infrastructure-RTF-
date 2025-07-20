# RTF Infrastructure - Multi-stage Docker Build
# Optimized for production deployment with security and performance

# ===== BUILD STAGE =====
FROM rust:1.70-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY backend/*/Cargo.toml ./backend/
COPY utils/*/Cargo.toml ./utils/
COPY infrastructure/*/Cargo.toml ./infrastructure/

# Create dummy source files to build dependencies
RUN mkdir -p backend/api/src \
    backend/bridge-defense/src \
    backend/compliance/src \
    backend/cross-chain/src \
    backend/emergency-handler/src \
    backend/esg-compliance/src \
    backend/exposure-detector/src \
    backend/governance/src \
    backend/llm-agent/src \
    backend/metrics/src \
    backend/monitoring/src \
    backend/oracle/src \
    backend/treasury/src \
    backend/zk-nav/src \
    utils/crypto/src \
    utils/post-quantum/src \
    utils/zk-proofs/src \
    infrastructure/deployment/src \
    infrastructure/monitoring/src

# Create dummy main.rs files
RUN echo "fn main() {}" > backend/api/src/main.rs
RUN echo "fn main() {}" > backend/bridge-defense/src/lib.rs
RUN echo "fn main() {}" > backend/compliance/src/lib.rs
RUN echo "fn main() {}" > backend/cross-chain/src/lib.rs
RUN echo "fn main() {}" > backend/emergency-handler/src/lib.rs
RUN echo "fn main() {}" > backend/esg-compliance/src/lib.rs
RUN echo "fn main() {}" > backend/exposure-detector/src/lib.rs
RUN echo "fn main() {}" > backend/governance/src/lib.rs
RUN echo "fn main() {}" > backend/llm-agent/src/lib.rs
RUN echo "fn main() {}" > backend/metrics/src/lib.rs
RUN echo "fn main() {}" > backend/monitoring/src/lib.rs
RUN echo "fn main() {}" > backend/oracle/src/lib.rs
RUN echo "fn main() {}" > backend/treasury/src/lib.rs
RUN echo "fn main() {}" > backend/zk-nav/src/lib.rs
RUN echo "fn main() {}" > utils/crypto/src/lib.rs
RUN echo "fn main() {}" > utils/post-quantum/src/lib.rs
RUN echo "fn main() {}" > utils/zk-proofs/src/lib.rs
RUN echo "fn main() {}" > infrastructure/deployment/src/lib.rs
RUN echo "fn main() {}" > infrastructure/monitoring/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --workspace
RUN rm -rf backend/*/src utils/*/src infrastructure/*/src

# Copy actual source code
COPY backend/ ./backend/
COPY utils/ ./utils/
COPY infrastructure/ ./infrastructure/
COPY config/ ./config/

# Build the actual application
RUN cargo build --release --workspace

# ===== RUNTIME STAGE =====
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r rtf && useradd -r -g rtf rtf

# Set working directory
WORKDIR /app

# Copy built binaries from builder stage
COPY --from=builder /app/target/release/rtf-* ./bin/
COPY --from=builder /app/config/ ./config/

# Copy scripts
COPY scripts/ ./scripts/
RUN chmod +x scripts/*.sh

# Create necessary directories
RUN mkdir -p logs data tmp \
    && chown -R rtf:rtf /app

# Switch to non-root user
USER rtf

# Expose ports
EXPOSE 8000 8001 8002 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Default command
CMD ["./bin/rtf-api"]

# ===== DEVELOPMENT STAGE =====
FROM builder as development

# Install additional development tools
RUN cargo install cargo-watch cargo-audit cargo-tarpaulin

# Copy source code
COPY . .

# Set development environment
ENV RUST_LOG=debug
ENV ENVIRONMENT=development

# Expose additional ports for development
EXPOSE 8000 8001 8002 9090 9091

# Development command with hot reload
CMD ["cargo", "watch", "-x", "run"]

# ===== TESTING STAGE =====
FROM builder as testing

# Copy test files
COPY tests/ ./tests/

# Run tests
RUN cargo test --release --workspace

# Run security audit
RUN cargo audit

# Generate coverage report
RUN cargo tarpaulin --all-features --workspace --timeout 120

# ===== PRODUCTION STAGE =====
FROM runtime as production

# Production-specific configurations
ENV RUST_LOG=info
ENV ENVIRONMENT=production

# Copy production configuration
COPY config/production.toml ./config/

# Set resource limits
USER rtf
WORKDIR /app

# Production command
CMD ["./bin/rtf-api", "--config", "config/production.toml"]
