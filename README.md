# Rustodon
[![Build Status](https://travis-ci.org/rustodon/rustodon.svg?branch=master)](https://travis-ci.org/rustodon/rustodon) [![dependency status](https://deps.rs/repo/github/rustodon/rustodon/status.svg)](https://deps.rs/repo/github/rustodon/rustodon) ![maintainance: actively developed](https://img.shields.io/badge/maintenance-actively%20developed-brightgreen.svg) [![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/rustodon/rustodon.svg)](http://isitmaintained.com/project/rustodon/rustodon "Average time to resolve an issue") [![Percentage of issues still open](http://isitmaintained.com/badge/open/rustodon/rustodon.svg)](http://isitmaintained.com/project/rustodon/rustodon "Percentage of issues still open") [![All Contributors](https://img.shields.io/badge/all_contributors-13-orange.svg?style=flat-square)](#contributors)


Rustodon is a [Mastodon](https://joinmastodon.org)-compatible _federated social microblogging server_. It utilizes [_ActivityPub_](http://activitypub.rocks) to _federate_ with a constellation of _other servers_, connecting their communities with yours.

## Current Status
**You probably don't want to use this, yet**. Federation is WIP, etc.

We currently have authentication, users, profiles, statuses, content warnings, actors and statuses published as both HTML and AS2, and timelines.
We **do not** have a job system, status delivery, inboxes, outboxes, notifications, mentions, post privacy, or account privacy.

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

Once you have installed these base components, you should run `scripts/setup` to install the remainder of the application dependencies.

### Building the browser-side code

To compile SCSS, and minify CSS, JS and SVG resources, we use [Gulp](https://gulpjs.com/). If you intend to make changes to these files, you will need to install [Node](https://nodejs.org/) and [NPM](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm). Then, in the Rustodon code folder, run these commands to install and run Gulp:
```bash
sudo npm install gulp-cli -g
npm install
gulp
```

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

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore -->
<table><tr><td align="center"><a href="https://imer.in"><img src="https://avatars3.githubusercontent.com/u/20133857?v=4" width="100px;" alt="Erin Moon"/><br /><sub><b>Erin Moon</b></sub></a><br /><a href="#blog-barzamin" title="Blogposts">📝</a> <a href="#content-barzamin" title="Content">🖋</a> <a href="https://github.com/rustodon/rustodon/commits?author=barzamin" title="Code">💻</a> <a href="#ideas-barzamin" title="Ideas, Planning, & Feedback">🤔</a> <a href="#maintenance-barzamin" title="Maintenance">🚧</a> <a href="#review-barzamin" title="Reviewed Pull Requests">👀</a> <a href="https://github.com/rustodon/rustodon/commits?author=barzamin" title="Documentation">📖</a></td><td align="center"><a href="https://measlytwerp.live"><img src="https://avatars2.githubusercontent.com/u/42093217?v=4" width="100px;" alt="Measly Twerp"/><br /><sub><b>Measly Twerp</b></sub></a><br /><a href="https://github.com/rustodon/rustodon/commits?author=measlytwerp" title="Code">💻</a> <a href="#ideas-measlytwerp" title="Ideas, Planning, & Feedback">🤔</a></td><td align="center"><a href="https://gitlab.peach-bun.com/yipdw"><img src="https://avatars3.githubusercontent.com/u/3859?v=4" width="100px;" alt="David Yip"/><br /><sub><b>David Yip</b></sub></a><br /><a href="https://github.com/rustodon/rustodon/commits?author=yipdw" title="Code">💻</a> <a href="#ideas-yipdw" title="Ideas, Planning, & Feedback">🤔</a></td><td align="center"><a href="http://www.robot-disco.net"><img src="https://avatars1.githubusercontent.com/u/487847?v=4" width="100px;" alt="Gaelan D'costa"/><br /><sub><b>Gaelan D'costa</b></sub></a><br /><a href="#infra-RobotDisco" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a> <a href="#ideas-RobotDisco" title="Ideas, Planning, & Feedback">🤔</a></td><td align="center"><a href="http://yeti-factory.org/"><img src="https://avatars0.githubusercontent.com/u/3809?v=4" width="100px;" alt="Chris Zelenak"/><br /><sub><b>Chris Zelenak</b></sub></a><br /><a href="https://github.com/rustodon/rustodon/commits?author=netshade" title="Code">💻</a> <a href="https://github.com/rustodon/rustodon/commits?author=netshade" title="Documentation">📖</a></td><td align="center"><a href="https://github.com/y6nH"><img src="https://avatars0.githubusercontent.com/u/355120?v=4" width="100px;" alt="Hugh"/><br /><sub><b>Hugh</b></sub></a><br /><a href="https://github.com/rustodon/rustodon/commits?author=y6nH" title="Code">💻</a> <a href="#design-y6nH" title="Design">🎨</a> <a href="#content-y6nH" title="Content">🖋</a> <a href="#ideas-y6nH" title="Ideas, Planning, & Feedback">🤔</a></td><td align="center"><a href="https://heiber.im"><img src="https://avatars2.githubusercontent.com/u/616813?v=4" width="100px;" alt="Moritz Heiber"/><br /><sub><b>Moritz Heiber</b></sub></a><br /><a href="https://github.com/rustodon/rustodon/commits?author=moritzheiber" title="Documentation">📖</a> <a href="#platform-moritzheiber" title="Packaging/porting to new platform">📦</a> <a href="#tool-moritzheiber" title="Tools">🔧</a></td></tr><tr><td align="center"><a href="https://linuxwit.ch"><img src="https://avatars2.githubusercontent.com/u/52814?v=4" width="100px;" alt="iliana destroyer of worlds"/><br /><sub><b>iliana destroyer of worlds</b></sub></a><br /><a href="https://github.com/rustodon/rustodon/commits?author=iliana" title="Code">💻</a></td><td align="center"><a href="https://github.com/1011X"><img src="https://avatars0.githubusercontent.com/u/1851619?v=4" width="100px;" alt="1011X"/><br /><sub><b>1011X</b></sub></a><br /><a href="#maintenance-1011X" title="Maintenance">🚧</a></td><td align="center"><a href="https://www.csos95.com"><img src="https://avatars0.githubusercontent.com/u/1892750?v=4" width="100px;" alt="Christopher Silva"/><br /><sub><b>Christopher Silva</b></sub></a><br /><a href="#maintenance-csos95" title="Maintenance">🚧</a> <a href="https://github.com/rustodon/rustodon/commits?author=csos95" title="Code">💻</a></td><td align="center"><a href="https://www.utam0k.jp/"><img src="https://avatars3.githubusercontent.com/u/13010913?v=4" width="100px;" alt="utam0k"/><br /><sub><b>utam0k</b></sub></a><br /><a href="#maintenance-utam0k" title="Maintenance">🚧</a> <a href="#platform-utam0k" title="Packaging/porting to new platform">📦</a></td><td align="center"><a href="https://github.com/dexamphetamine"><img src="https://avatars2.githubusercontent.com/u/20431955?v=4" width="100px;" alt="dexamphetamine"/><br /><sub><b>dexamphetamine</b></sub></a><br /><a href="https://github.com/rustodon/rustodon/commits?author=dexamphetamine" title="Code">💻</a> <a href="#ideas-dexamphetamine" title="Ideas, Planning, & Feedback">🤔</a></td><td align="center"><a href="https://ktn.fyi"><img src="https://avatars1.githubusercontent.com/u/9281956?v=4" width="100px;" alt="ash lea"/><br /><sub><b>ash lea</b></sub></a><br /><a href="https://github.com/rustodon/rustodon/commits?author=ashkitten" title="Code">💻</a> <a href="#ideas-ashkitten" title="Ideas, Planning, & Feedback">🤔</a> <a href="#maintenance-ashkitten" title="Maintenance">🚧</a></td></tr></table>

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!
