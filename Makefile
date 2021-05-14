pre-commit:
	cargo fmt -- --check
	cargo test -- --test-threads 1
	cargo clippy -- -D clippy::all

release:
	cargo build --release

# Sadly, this misses coverage for those integrations tests that use
# assert_cmd, as it does not follow forks
coverage:
	cargo tarpaulin -- --test-threads 1

# TODO: add trigger for this target from within github actions
codecov:
	cargo tarpaulin --out Xml -- --test-threads 1
	codecov-io
