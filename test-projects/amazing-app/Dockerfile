# Multi-stage build for {{project-name}}
# This Dockerfile creates an optimized production image

# Stage 1: Build dependencies
FROM rust:1.75-slim as chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# Stage 2: Prepare recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies
FROM chef AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    {%- if database == "postgres" %}
    libpq-dev \
    {%- elif database == "mysql" %}
    default-libmysqlclient-dev \
    {%- elif database == "sqlite" %}
    libsqlite3-dev \
    {%- endif %}
    && rm -rf /var/lib/apt/lists/*

# Build dependencies (this step is cached)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release

# Stage 4: Runtime image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    {%- if database == "postgres" %}
    libpq5 \
    {%- elif database == "mysql" %}
    default-mysql-client \
    {%- elif database == "sqlite" %}
    libsqlite3-0 \
    {%- endif %}
    && rm -rf /var/lib/apt/lists/*

# Create application user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create application directory
WORKDIR /app

# Copy application binary
COPY --from=builder /app/target/release/{{project-name}}-server /app/{{project-name}}-server

# Copy static assets
COPY --from=builder /app/static /app/static

{%- if database != "none" %}
# Copy migrations
COPY --from=builder /app/migrations /app/migrations
{%- endif %}

# Create data directory for SQLite or uploads
{%- if database == "sqlite" %}
RUN mkdir -p /app/data && chown -R appuser:appuser /app/data
{%- endif %}
RUN mkdir -p /app/uploads && chown -R appuser:appuser /app/uploads

# Set permissions
RUN chown -R appuser:appuser /app
RUN chmod +x /app/{{project-name}}-server

# Switch to non-root user
USER appuser

# Expose port
EXPOSE {{port}}

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:{{port}}/health || exit 1

# Environment variables
ENV RUST_LOG={{crate_name}}=info
ENV HOST=0.0.0.0
ENV PORT={{port}}
ENV ENVIRONMENT=production

# Start the application
CMD ["/app/{{project-name}}-server"]