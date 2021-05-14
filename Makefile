pre-commit:
	cargo fmt -- --check
	cargo test -- --test-threads 1
	cargo clippy -- -D clippy::all

# Sadly, this misses coverage for those integrations tests that use
# assert_cmd, as it does not follow forks
coverage:
	cargo tarpaulin -- --test-threads 1

release:
	cargo build --release
