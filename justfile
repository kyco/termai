build:
	cargo build --release

run args:
	cargo run --release -- {{args}}

test:
	cargo test

test-quiet:
	cargo test --quiet

test-coverage:
	cargo tarpaulin --skip-clean --fail-under 0 --out Stdout

clean:
	cargo clean

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

update-deps:
	cargo update

release:
	cargo release

outdated:
	cargo outdated

doc:
	cargo doc --open