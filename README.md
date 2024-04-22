<h1 align="center"> cpkg </h1>

<p align="center">
	A dead simple, modern package manager for C.
</p>

<div align="center">
	<a href="https://github.com/DvvCz/cpkg/actions">
		<img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/DvvCz/cpkg/nightly.yml?label=nightly">
	</a>
	<a href="https://crates.io/crates/cpkg">
		<img alt="Crates.io Version" src="https://img.shields.io/crates/v/cpkg">
	</a>
	<a href="https://github.com/DvvCz/cpkg/releases/latest">
		<img alt="GitHub Release" src="https://img.shields.io/github/v/release/DvvCz/cpkg">
	</a>
</div>

## What is cpkg?

`cpkg` is an all-in-one wrapper for tools like `gcc`, `clang`, `doxygen` and `clang-format`.  
It automatically detects which are present on your system, allowing you to use them with one simple cli.

Inspired by the convenience of modern tools like `cargo` and `bun`.

## Usage

```bash
cpkg init
cpkg run
```

## Features

### 🧑‍💻 Project Runner

You can create a project with `new` or `init`, and then run `/src/main.c` with `cpkg run` or `cpkg build`.

You can run tests located in `/src/*.test.c` and `/tests/*.c` with `cpkg test`.

### 📦 Package Management

You can add local paths with `cpkg add <name> --path /path/to/dependency` and git dependencies with `cpkg add <name> --git https://github.com/nothings/stb/tree/master`.

### 🗄️ Project File Generation

Project files can be generated using `cpkg generate`.

This creates a project file that acts as if you ran `cpkg build`, without `cpkg`.

*Currently only supports basic [`Makefile`](https://www.gnu.org/software/make) generation*

### 🛠️ Other Components

`cpkg` supports other functionalities:

* Formatting using [`clang-format`](https://clang.llvm.org/docs/ClangFormat.html)
* Documenting using [`doxygen`](https://www.doxygen.nl)

## ⏬ Installation

### 📩 Releases

You can download the `cpkg` binary from [the releases](https://github.com/DvvCz/cpkg/releases) (or a nightly build from [actions](https://github.com/DvvCz/cpkg/actions))

### 📦 Cargo

If you have `cargo` you can install from crates.io.

```
cargo install cpkg
```

Or clone the repository and install it locally.

```
git clone https://github.com/DvvCz/cpkg
cargo install --path cpkg
```

### 🔄 Upgrading

You can easily upgrade your `cpkg` binary using the `cpkg upgrade` command.