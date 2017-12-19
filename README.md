# Rustodon

## Hacking on the code
Rustodon depends on libraries (looking at you, Diesel and Rocket) that require bleeding-edge nightly rustc features. Ideally, install Rust via [`rustup`](https://www.rustup.rs/) and set an override in the Rustodon directory with
```
$ rustup override set nightly
```
