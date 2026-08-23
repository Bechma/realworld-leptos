ARG RUST_VERSION=1.98.0
ARG CARGO_LEPTOS_VERSION=0.3.7

FROM rust:${RUST_VERSION}-trixie AS builder

ARG CARGO_LEPTOS_VERSION

RUN curl --proto '=https' --tlsv1.2 -LsSf \
    "https://github.com/leptos-rs/cargo-leptos/releases/download/v${CARGO_LEPTOS_VERSION}/cargo-leptos-installer.sh" \
    | sh \
    && cargo leptos --version \
    && rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY . .

RUN cargo leptos build --release --precompress

FROM debian:trixie-slim AS runner

WORKDIR /app

COPY --from=builder /app/target/release/realworld-leptos /app/realworld-leptos
COPY --from=builder /app/target/site /app/site

ENV LEPTOS_OUTPUT_NAME="realworld-leptos"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"

EXPOSE 8080

# Required at runtime: DATABASE_URL and JWT_SECRET.
# Required for password-reset email: MAILER_EMAIL, MAILER_PASSWD, and MAILER_SMTP_SERVER.
CMD ["/app/realworld-leptos"]
