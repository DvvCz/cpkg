# cpkg


> A dead simple C package manager.

This is essentially an all-in-one wrapper for gcc, clang, doxygen, clang-format, etc.  
`cpkg` automatically detects which are present on your system, allowing you to use them with one simple cli.

Inspired by the convenience of `cargo` and `bun`.

## Usage

```
cpkg new hello_world
cd hello_world
cpkg run
```

## Features
- [x] `new`
- [x] `init`
- [x] `build`
- [x] `run`
- [x] `test`
- [x] `doc` w/ Doxygen
- [x] `repl`
- [x] `format`

## Installation

### Releases

You can download the `cpkg` binary from [the releases](https://github.com/DvvCz/cpkg/releases) (or a nightly build from [actions](https://github.com/DvvCz/cpkg/actions))

### Cargo

If you have `cargo` you can install from crates.io.

```
cargo install cpkg
```

Or clone the repository and install it locally.

```
git clone https://github.com/DvvCz/cpkg
cargo install --path cpkg
```