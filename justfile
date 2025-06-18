build:
	cargo build --release

run args:
	cargo run --release -- {{args}}

test:
	cargo test

test-quiet:
	cargo test --quiet

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