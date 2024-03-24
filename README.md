# cpkg

A dead simple C package manager.

It just creates and compiles projects for you, handling your respective backends (gcc, clang) automatically. Inspired by the convenience of Rust's `cargo`.

## Usage

```
cpkg new hello_world
cd hello_world
cpkg run
```

## Installation

Currently not on crates.io and not planned anytime soon.

You can install manually using these commands:

```
git clone https://github.com/DvvCz/cpkg
cargo install --path cpkg
```