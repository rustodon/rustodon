# Rustodon

[![Build Status](https://travis-ci.org/rustodon/rustodon.svg?branch=master)](https://travis-ci.org/rustodon/rustodon) [![dependency status](https://deps.rs/repo/github/rustodon/rustodon/status.svg)](https://deps.rs/repo/github/rustodon/rustodon)

Rustodon is an [Mastodon](https://joinmastodon.org)-compatible _federated social microblogging server_. It utilizes [_ActivityPub_](http://activitypub.rocks) to _federate_ with a constellation of _other servers_, connecting their communities with yours.

## Current Status
**You probably don't want to use this, yet**. Federation is WIP, UI is WIP, we don't have timelines, etc.

We currently have authentication, users, profiles, statuses, content warnings, actor and status visibility as both HTML and AS2.
We **do not** have timelines, status delivery, inboxes, outboxes, notifcations, mentions, post privacy, or account privacy.

If you want to work on making Rustodon feature-complete, check out the [issue tracker](https://github.com/rustodon/rustodon/issues)! We're not just looking for Rust devs, either; CSS witches, brainstormers, and documentation enthusiasts are highly welcome :smiley:

## Hacking on the code
Rustodon depends on libraries (looking at you, Diesel and Rocket) that require bleeding-edge nightly rustc features. Ideally, install Rust via [`rustup`](https://www.rustup.rs/) and set an override in the Rustodon directory with
```
$ rustup override set $(cat REQUIRED_RUST_NIGHTLY)
```

We use [Postgres](https://www.postgresql.org/) for data storage, so get a Postgres instance running, create a user, and set an environment variable `DATABASE_URL` to a Postgres URI, like so:
```
$ export DATABASE_URL=postgres://username:password@localhost/rustodon
```

This environment variable could alternatively be added to the `.env` file (you can use `git update-index --assume-unchanged .env` to keep Git from telling you `.env` has been modified. Please don't commit _your_ environment to the repo :p).

If you don't have a Postgres instance available, you can use the supplied [docker-compose](https://github.com/docker/compose/) configuration file to start an instance:

```
$ docker-compose up -d
```
The instance will be started in the background. The default username _and password_ is `rustodon`. The corresponding connection string would be
```
$ export DATABASE_URL=postgres://rustodon:rustodon@localhost/rustodon
```

On some operating systems, you may need to separately install the Postgres client library:

* Debian/Ubuntu/etc: `apt install libpq-dev`
* Arch: `pacman -S postgresql-libs`

Sass/SCSS is used to make stylesheeting a bit nicer, so you'll have to install Ruby via your favourite method and `bundle install`.

To set up a new database in Postgres and run all the migrations, first install the Diesel CLI:
```
# We only need PostgreSQL support
$ cargo install diesel_cli --no-default-features --features="postgres"
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
