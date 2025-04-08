# cargo-templates

Cargo templates for personal use.

### How to use

This is a selection of templates to be used alongside [Cargo Generate](https://cargo-generate.github.io/cargo-generate/). The way I intend to use this is to quickly set up PoCs or prototypes for things that I tend to do often and won't necessarily commit to long-term.

There is currently only one template (`evm/`). More to come!

### Instructions

`cargo generate --git git@github.com:CalabashSquash/cargo-templates.git <SUBDIRECTORY>/`

### `evm/`

Herein lies a few util functions for interacting with an evm chain. The common stuff. This includes:

- Querying chain data/blocks directly
- Reading from smart contracts
- Querying data historically.
