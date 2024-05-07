

# Omnity indexer

1. Modify the `DATABASE_URL` var in `.env` to point to your chosen database

1. Turn on the appropriate database feature for your chosen db in `service/Cargo.toml` (the `"sqlx-sqlite",` line)

1. Execute `cargo run` to start the server

1. Visit [localhost:3000/api/graphql](http://localhost:3000/api/graphql) in browser

Run mock test on the service logic crate:

```bash
cd service
cargo test --features mock
```
