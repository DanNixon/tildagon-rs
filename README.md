# Tildagon :heart: Rust

[![Crates.io Version](https://img.shields.io/crates/v/tildagon)](https://crates.io/crates/tildagon)

A Rust board support crate for the [Electromagnetic Field](https://www.emfcamp.org/) [Tildagon](https://tildagon.badge.emfcamp.org/) badge.

There are still some missing features, but very usable for badge creations that will not be reconfigured much.

## Development setup

Assumes using Nix dev shell and [Distrobox](https://distrobox.it/).
If you are using a "normal" Linux distro, you can probably skip the Distrobox steps (this was the easiest option on NixOS).

Initial setup:

- `direnv allow`
- `distrobox create`
- `espup install`

To activate development environment:

- `distrobox enter`
- `. $HOME/export-esp.sh`
