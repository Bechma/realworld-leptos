# Realworld app with Leptos + Axum + Postgres

You can check it online in https://realworld-leptos.onrender.com (it might take a couple of seconds to startup, we're in the free tier).

### WIP feature

- Add some test cases

# Requirements

## Rust with Webassembly support

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Once finished, add webassembly target to rust:

`rustup target add wasm32-unknown-unknown`

## cargo-leptos

This is an utility to easily compile either backend and frontend at the same time:

`cargo install cargo-leptos`

# How to run this project locally

First, deploy a local postgres database, maybe docker is the fastest solution:

`docker run --name postgres -e POSTGRES_PASSWORD=postgres -p 5432:5432 -d postgres`

Clone it into your machine and run it with cargo-leptos:

```
git clone https://github.com/Bechma/realworld-leptos.git
cd realworld-leptos
export DATABASE_URL=postgres://postgres:postgres@localhost/postgres
cargo leptos watch
```

# Run it with docker compose

You can also run the application in release mode using docker compose:

`docker compose up`

And navigate to http://localhost:8080/
