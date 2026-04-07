# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.5](https://github.com/DanNixon/tildagon-rs/compare/v0.0.4...v0.0.5) - 2026-04-07

### Added

- [**breaking**] upgrade to esp-hal 1.0.0

### Fixed

- derive Default where possible
- *(docs)* update incorrect reference to devenv.sh

### Other

- *(deps)* bump DeterminateSystems/nix-installer-action from 21 to 22
- update dependabot config
- add typos to CI and make corrections
- switch from devenv to nix flake
- *(deps)* bump actions/checkout from 5 to 6
- update devenv inputs

## [0.0.4](https://github.com/DanNixon/tildagon-rs/compare/v0.0.3...v0.0.4) - 2025-09-20

### Added

- support IMU
- i2c bus scanning

### Fixed

- compilation failure with DEFMT_LOG=debug

## [0.0.3](https://github.com/DanNixon/tildagon-rs/compare/v0.0.2...v0.0.3) - 2025-09-17

### Added

- multicore demo
- async i2c

### Fixed

- use blocking smartleds
- *(docs)* adjust markdown lint config

## [0.0.2](https://github.com/DanNixon/tildagon-rs/compare/v0.0.1...v0.0.2) - 2025-09-15

### Added

- add demo with no top board

### Other

- add crates.io badge to readme

## 0.0.1 - 2025-09-15

### Added

- initial commit
