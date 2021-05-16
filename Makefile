pre-commit:
	cargo fmt -- --check
	cargo test -- --test-threads 1
	cargo clippy -- -D clippy::all

release:
	cargo build --release

test:
	cargo test -- --test-threads 1

# Sadly, this misses coverage for those integrations tests that use
# assert_cmd, as it does not follow forks
coverage:
	cargo tarpaulin -o Html -- --test-threads 1
