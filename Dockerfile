FROM rust:1.71-bookworm as builder

RUN cargo install cargo-leptos &&\
    rustup target add wasm32-unknown-unknown &&\
    mkdir -p /app

WORKDIR /app
ENV JWT_SECRET="replaceme when ran in prod"
COPY . .

RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/root/.cache \
    sed -i 's/env = "DEV"/env = "PROD"/' ./Cargo.toml &&\
    cargo leptos build -r -vv

FROM debian:bookworm-slim as runner

WORKDIR /app

COPY --from=builder /app/target/server/release/realworld-leptos /app/
COPY --from=builder /app/target/site /app/site

ENV LEPTOS_OUTPUT_NAME="realworld-leptos"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"

EXPOSE 8080

CMD ["/app/realworld-leptos"]