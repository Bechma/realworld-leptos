# Realworld app with Leptos + Axum + Postgres

You can check it online in https://realworld-leptos.onrender.com (it might take a couple of seconds to startup).

# Requirements

## Rust with Webassembly support

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Once finished, add webassembly target to rust:

`rustup target add wasm32-unknown-unknown`

## cargo-leptos

This is an utility to easily compile either backend and frontend at the same time:

`cargo install cargo-leptos`

# sqlx

We need to run migrations before compiling, otherwise the query! macros will fail and abort the compilation:

`cargo install sqlx-cli`

# How to run this project locally

First, deploy a local postgres database, maybe docker is the fastest solution:

`docker run --name postgres -e POSTGRES_PASSWORD=postgres -p 5432:5432 -d postgres`

Clone it into your machine and run it with cargo-leptos:

```
git clone https://github.com/Bechma/realworld-leptos.git
cd realworld-leptos
cp .env.example .env
source .env
cargo sqlx migrate run
cargo leptos watch
```

Change the placeholder in .env for `JWT_SECRET` for security reasons.

Also, there's a password reset functionality that works with a sending email. If you want
to use that feature you can set MAILER_EMAIL and MAILER_PASSWD with your email creds
and MAILER_SMTP_SERVER with your SMTP service.

# How to test this project

You will need to have a local database, in order to execute end2end testing.

```
cd end2end/
npm i
npx playwright install
cd ../
cargo leptos end-to-end
```

You will need to install the playright dependency in the end2end directory and
install the playwright drivers. With cargo-leptos the tests will be executed.

# Run it with docker compose

You can also run the application in release mode using docker compose:

`docker compose up`

And navigate to http://localhost:8080/

# Details about deployment

The deployment has been done thanks to the free tier of:

- https://render.com/ for the fullstack application
- https://neon.tech/ for the database (render has a 90 days expiration in the db)

Previously, I deployed the db in https://www.elephantsql.com/ ,
but [It will reach EOL soon](https://www.elephantsql.com/blog/end-of-life-announcement.html)