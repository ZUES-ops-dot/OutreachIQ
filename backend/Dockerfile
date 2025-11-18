FROM rust:1.79-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the actual application
RUN touch src/main.rs
RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache ca-certificates && \
    addgroup -g 1000 appgroup && \
    adduser -u 1000 -G appgroup -D appuser

WORKDIR /app

COPY --from=builder /app/target/release/outreachiq /app/outreachiq
COPY --from=builder /app/migrations /app/migrations

RUN chown -R appuser:appgroup /app

USER appuser

EXPOSE 8080

CMD ["./outreachiq"]
