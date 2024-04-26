# Developement

To start developing on the project, you will first need to install [rustup](https://rustup.rs/).

Rustup is an installer for the Rust language. It is the recommended way to install Rust, as it manages both the installation and updates of the Rust compiler and tools. It installs the Rust compiler (`rustc`) and the `cargo` package manager.

After installing `rustup`, you can install [`trunk`](https://trunkrs.dev/) with :

```bash
cargo install trunk
```

`trunk` allows you to build, bundle, and optimize your WebAssembly application.

Finally, you can clone the repository with :

```bash
git clone git@github.com:tramex/tramex.git
```

## Development

To start the application, you can run the following command in the repository :

```bash
# for the web application
trunk serve

# for the desktop application
cargo run
```
