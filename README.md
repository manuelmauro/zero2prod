# Rust Backend Stack

## Setup

### Installing Rust and Cargo

Install Rust as described [here](https://doc.rust-lang.org/book/ch01-01-installation.html).

### Installing `sqlx-cli`

SQLx is an async, pure Rust SQL crate featuring compile-time checked queries without a DSL.

SQLx-CLI is SQLx's associated command-line utility for managing databases, migrations, and enabling "offline" mode with sqlx::query!() and friends.
It is published on the Cargo crates registry as `sqlx-cli` and can be installed like so:

```shell
cargo install sqlx-cli --features postgres
```

### Running Postgres

The following script will start the latest version of Postgres using [Docker], create the database and run the migrations.

```shell
./scripts/init_db.sh
```

### Preparing SQLx data

There are 3 steps to building with "offline mode":

- Enable the SQLx's Cargo feature offline
  - E.g. in your Cargo.toml, sqlx = { features = [ "offline", ... ] }
- Save query metadata for offline usage
  - `cargo sqlx prepare`
- Build

### Starting the Application

With everything else set up, all you need to do at this point is:

```shell
cargo run
```

If successful, the API server is now listening at port 8080.

#### Hot Reload

Use [`cargo-watch`](https://crates.io/crates/cargo-watch) for hot reloading the server.

```bash
cargo watch -x run
```

### Preparing Tailwind CSS

#### Install NodeJS

Follow the documentation [here](https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating)

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
```

then install node with:

```bash
nvm install node
```

#### Install dependencies

```bash
npm run install
```

#### Build the project's CSS

```bash
npx tailwindcss -i ./input.css -o ./assets/output.css --watch
```

## Quality Assurance

### Testing

Run unit tests with:

```bash
cargo test
```

### Formatting

Format with:

```bash
cargo fmt --check
```

### Linting

Lint with:

```bash
cargo clippy -- -D warnings
```

### Code Coverage

Check code coverage:

```bash
cargo tarpaulin --verbose --workspace
```

## Deployment

### Dependencies

This application relies on Digital Ocean's [App Platform] for its CD pipeline.
Install Digital Ocean's CLI [doctl] to interact with platform.

### First Deployment

To deploy a new instance of the application on Digital Ocean run:

```bash
doctl apps create --spec spec.yaml
```

then migrate the production database with:

```bash
DATABASE_URL=<connection-string> sqlx migrate run
```

## License

Licensed under MIT license. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, shall be licensed as above, without any additional terms or conditions.

[Docker]: https://www.docker.com/
[App Platform]: https://docs.digitalocean.com/products/app-platform/
[doctl]: https://docs.digitalocean.com/reference/doctl/how-to/install/
