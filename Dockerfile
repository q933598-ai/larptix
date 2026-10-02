FROM node:24-bookworm-slim AS matrix-crypto
WORKDIR /matrix-crypto
RUN npm pack --silent @matrix-org/matrix-sdk-crypto-wasm@18.9.0 \
    && mkdir pkg \
    && tar -xzf *.tgz -C pkg --strip-components=1

FROM rust:bookworm AS builder
WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --locked --release -p larptrix-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home --home-dir /app larptrix

WORKDIR /app
COPY --from=builder /build/target/release/larptrix-server /usr/local/bin/larptrix-server
COPY client ./client
COPY --from=matrix-crypto /matrix-crypto/pkg /app/client/matrix-crypto-pkg
RUN mkdir -p /app/data/uploads && chown -R larptrix:larptrix /app

USER larptrix
ENV LARPTRIX_BIND=0.0.0.0:8080 \
    LARPTRIX_DB=/app/data/larptrix.db \
    LARPTRIX_CLIENT=/app/client \
    LARPTRIX_UPLOADS=/app/data/uploads \
    LARPTRIX_COOKIE_SECURE=1

EXPOSE 8080
CMD ["/usr/local/bin/larptrix-server"]