# Tildagon :heart: Rust

[![Crates.io Version](https://img.shields.io/crates/v/tildagon)](https://crates.io/crates/tildagon)

A Rust board support crate for the [Electromagnetic Field](https://www.emfcamp.org/) [Tildagon](https://tildagon.badge.emfcamp.org/) badge.

Still quite early in development and things might change a bit, but very usable for badge creations that will not be reconfigured much.

## Features

- [x] "Low speed" IO driver
- [x] Hexpansion ports
- [x] 2024 top board
   - [x] LEDs
   - [x] Buttons
   - [x] Display
- [x] IMU
- [ ] Power management
- [ ] Hexpansion metadata read(/write)
- [ ] Some form of dynamic hexpansion slot use

## Cargo features

- `top-board-none`: no support for any devices on the top board flat flex connection
- `top-board-2024`: support for the devices on the top board from EMF 2024

You will need to enable exactly one `top-board-*` feature.

## Development setup

Assumes using [devenv](https://devenv.sh/) and [Distrobox](https://distrobox.it/).
If you are using a "normal" Linux distro, you can probably skip the Distrobox steps (this was the easiest option on NixOS).

Initial setup:

- `direnv allow`
- `distrobox create`
- `espup install`

To activate development environment:

- `distrobox enter`
- `. ./.export-esp.sh`
