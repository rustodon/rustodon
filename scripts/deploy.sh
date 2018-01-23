#!/usr/bin/env bash

cd /srv/rustodon
/root/.cargo/bin/diesel migration run
