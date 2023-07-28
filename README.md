<picture>
    <source srcset="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_Solid_White.svg" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_RGB.svg" alt="Leptos Logo">
</picture>

# Missing features

- [ ] Fix Editor component when he receives a slug in the url
- [ ] Do some general manual testing
- [ ] Add some testing examples
- [ ] Deployment(maybe in Docker)

# Requirements

## Rust with Webassembly support

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Once finished, add webassembly target to rust:

`rustup target add wasm32-unknown-unknown`

## cargo-leptos

This is an utility to easily compile either backend and frontend at the same time:

`cargo install cargo-leptos`

# How to run this project

First, deploy a local postgres database, maybe docker is the fastest solution:

`docker run --name postgres -e POSTGRES_PASSWORD=postgres -p 5432:5432 -d postgres`

Clone it into your machine and run it with cargo-leptos:

```
git clone https://github.com/Bechma/realworld-leptos.git
cd realworld-leptos
export DATABASE_URL=postgres://postgres:postgres@localhost/postgres
cargo leptos watch
```

## Installing Additional Tools

By default, `cargo-leptos` uses `nightly` Rust, `cargo-generate`, and `sass`. If you run into any trouble, you may need to install one or more of these tools.

1. `rustup toolchain install nightly --allow-downgrade` - make sure you have Rust nightly
2. `rustup default nightly` - setup nightly as default, or you can use rust-toolchain file later on
3. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
4. `cargo install cargo-generate` - install `cargo-generate` binary (should be installed automatically in future)
5. `npm install -g sass` - install `dart-sass` (should be optional in future
