<!-- markdownlint-disable MD033 -->

# ⚙️ Installation

The preferred way of installing Lune is using [Aftman](https://github.com/lpghatguy/aftman).

Running this command in your terminal will add `lune` to an `aftman.toml` file in the current directory, or create one if it does not exist:

```sh
aftman add filiptibell/lune
```

## Other options

### Building from source

Building and installing from source requires the latest version of [Rust & Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) to be installed on your system. <br />
Once installed, run the following command in your terminal:

```sh
cargo install lune --locked
```

Note that Lune does not make any minimum supported rust version (MSRV) guarantees and you may need to upgrade your version of Rust to update Lune in the future.

### Using GitHub Releases

You can download pre-built binaries for most systems directly from the [GitHub Releases](https://github.com/filiptibell/lune/releases) page. <br />
There are many tools that can install directly from releases, and it is up to you to choose what tool to use when installing using this method.

## Next steps

Now that you've installed Lune, head over to the [Writing Scripts](https://github.com/filiptibell/lune/wiki/Getting-Started---2-Writing-Scripts) page to write your first script!
