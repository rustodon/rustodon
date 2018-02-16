# Rustodon

[![Build Status](https://travis-ci.org/rustodon/rustodon.svg?branch=master)](https://travis-ci.org/rustodon/rustodon)

## Hacking on the code
Rustodon depends on libraries (looking at you, Diesel and Rocket) that require bleeding-edge nightly rustc features. Ideally, install Rust via [`rustup`](https://www.rustup.rs/) and set an override in the Rustodon directory with
```
$ rustup override set nightly
```

We use [Postgres](https://www.postgresql.org/) for data storage, so get a Postgres instance running, create a user, and set an environment variable `DATABASE_URL` to a Postgres URI, like so:
```
$ export DATABASE_URL=postgres://username:password@localhost/rustodon
```

This environment variable could alternatively be added to the `.env` file (you can use `git update-index --assume-unchanged .env` to keep Git from telling you `.env` has been modified. Please don't commit _your_ environment to the repo :p).

On some operating systems, you may need to separately install the Postgres client library, as well as the MySQL library (even if we won't ever use it):

* Debian/Ubuntu: `apt install libpq-dev libmysqlclient-dev`

Sass/SCSS is used to make stylesheeting a bit nicer, so you'll have to install Ruby via your favourite method and `gem install sass`.

To set up a new database in Postgres and run all the migrations, first install the Diesel CLI:
```
$ cargo install diesel_cli
```

Cargo, by default, will install any package binaries into `~/.cargo/bin`. We will assume you have added that directory to your `PATH` environment variable.

Then, run the database setup:
```
$ diesel database setup
```

You can now launch Rustodon by running
```
$ cargo run
```

Rustodon will launch on `http://localhost:8000` by default; this can be overriden by setting [certain environment variables](https://rocket.rs/guide/configuration/#environment-variables).

Federation requires that the application know where it's hosted, and (thanks to Webfinger) also forces us to serve over HTTPS. To get around this in a development environment, you can use [ngrok](https://ngrok.com/) or a similar service. To make sure the app knows where it's serving from (used to compute, eg, AS2 UIDs), remember to set `DOMAIN` in `.env` (again, the `--assume-unchanged` trick is very useful).
