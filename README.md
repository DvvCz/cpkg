# cpkg


> A dead simple C package manager.

This is essentially an all-in-one wrapper for gcc, clang, doxygen, clang-format, etc.

`cpkg` automatically detects which are present on your system, allowing you to use them with one simple cli.

Inspired by the convenience of modern tools like `cargo` and `bun`.

## Usage

```bash
cpkg init
cpkg run
```

## Features

### Project Runner

You can create a project with `new` or `init`, and then run `/src/main.c` with `cpkg run` or `cpkg build`

You can run tests located in `/src/*.test.c` and `/tests/*.c` with `cpkg test`.

### Package Management

You can add local paths with `cpkg add <name> --path /path/to/dependency` and git dependencies with `cpkg add <name> --git https://github.com/nothings/stb/tree/master`.

### Other Components

`cpkg` supports other functionalities:

* Formatting using [`clang-format`](https://clang.llvm.org/docs/ClangFormat.html)
* Documenting using [`doxygen`](https://www.doxygen.nl)

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

### Upgrading

You can easily upgrade your `cpkg` binary using the `cpkg upgrade` command.