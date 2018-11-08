# Rustodon

[![Build Status](https://travis-ci.org/rustodon/rustodon.svg?branch=master)](https://travis-ci.org/rustodon/rustodon) [![dependency status](https://deps.rs/repo/github/rustodon/rustodon/status.svg)](https://deps.rs/repo/github/rustodon/rustodon) [![chat on Zulip](https://img.shields.io/badge/chat%20on-zulip-459f7d.svg)](https://rustodon.z.hecke.rs/)

Rustodon is an [Mastodon](https://joinmastodon.org)-compatible _federated social microblogging server_. It utilizes [_ActivityPub_](http://activitypub.rocks) to _federate_ with a constellation of _other servers_, connecting their communities with yours.

## Current Status
**You probably don't want to use this, yet**. Federation is WIP, etc.

We currently have authentication, users, profiles, statuses, content warnings, actors and statuses published as both HTML and AS2, and timelines.
We **do not** have a job system, status delivery, inboxes, outboxes, notifcations, mentions, post privacy, or account privacy.

If you want to work on making Rustodon feature-complete, check out the [issue tracker](https://github.com/rustodon/rustodon/issues)! We're not just looking for Rust devs, either; CSS witches, documentarians, UI/UX aficionados, etc, are highly welcome :smiley:

## Hacking on the code

You will need to install several base dependencies:

1. [Rust](https://www.rust-lang.org/en-US/install.html). Make sure you have followed the official instructions regarding your `PATH` variable.
   > In the Rust development environment, all tools are installed to the ~/.cargo/bin directory, and this is where you will find the Rust toolchain, including rustc, cargo, and rustup.
   > Accordingly, it is customary for Rust developers to include this directory in their PATH environment variable. During installation rustup will attempt to configure the PATH. Because of differences between platforms, command shells, and bugs in rustup, the modifications to PATH may not take effect until the console is restarted, or the user is logged out, or it may not succeed at all.
   > If, after installation, running rustc --version in the console fails, this is the most likely reason.
1. [Postgres](https://www.postgresql.org/download/). If you don't have a Postgres instance available, you can use the supplied [docker-compose](https://github.com/docker/compose/) configuration file to start an instance:
   ```
   docker-compose up -d
   ```
   The instance will be started in the background. The default username _and password_ is `rustodon`. The corresponding connection string would be:
   ```
   export DATABASE_URL=postgres://rustodon:rustodon@localhost/rustodon
   ```
   On some operating systems, you may need to separately install the Postgres client library:
   * Debian/Ubuntu/etc: `apt install libpq-dev`
   * Arch: `pacman -S postgresql-libs`
1. [SASS](https://sass-lang.com/). To install, follow [the official directions](https://sass-lang.com/install), and make sure the `sass` binary is somewhere in your `PATH`.

Once you have installed these base components, you should run `scripts/setup` to install the remainder of the application dependencies.

### Running the application

To run the application once you have installed all dependencies, you should run either:

* `cargo run`: Runs just the server
* `fors start`: Runs the server and additional helper processes

Rustodon will launch on `http://localhost:8000` by default; this can be overriden by setting [certain environment variables](https://rocket.rs/guide/configuration/#environment-variables).

Federation requires that the application know where it's hosted, and (thanks to Webfinger) also forces us to serve over HTTPS. To get around this in a development environment, you can use [ngrok](https://ngrok.com/) or a similar service. To make sure the app knows where it's serving from (used to compute, eg, AS2 UIDs), set `DOMAIN` in `.env`.

### Running database migrations

`diesel database setup`

### Running the tests

`cargo test --all`
