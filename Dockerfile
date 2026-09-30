# ponytail: 1.80's Cargo can't parse edition2024 manifests in current deps (idna_adapter) — match local toolchain that builds the lockfile.
FROM rust:1.98-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

# ponytail: distroless/cc instead of scratch — sqlite needs libc; scratch would require a musl static build.
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/aaas-v2 /aaas-v2
COPY --from=builder /app/migrations /migrations

VOLUME ["/data", "/secrets"]
EXPOSE 3000

ENV PORT=3000
ENV DATABASE_URL=/data/aaas-v2.db
ENV RUST_LOG=info

ENTRYPOINT ["/aaas-v2"]
