FROM rust:1.78-bookworm as builder

RUN cargo install cargo-leptos &&\
    rustup target add wasm32-unknown-unknown &&\
    mkdir -p /app

WORKDIR /app
ENV JWT_SECRET="replaceme when ran in prod"
COPY . .

RUN cargo leptos build -r -vv

FROM debian:bookworm-slim as runner

WORKDIR /app

COPY --from=builder /app/target/release/realworld-leptos /app/realworld-leptos
COPY --from=builder /app/target/site /app/site

ENV LEPTOS_OUTPUT_NAME="realworld-leptos"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"

EXPOSE 8080

# Remember to set JWT_SECRET and DATABASE_URL environmental variables
CMD ["/app/realworld-leptos"]