
all: build

build:
	cargo build --release

test:
	cargo test --verbose

run:
	cargo run --release

clean:
	cargo clean
