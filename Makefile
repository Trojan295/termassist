
.PHONY: compile install

compile:
	cargo build --release

install:
	sudo cp target/release/termassist /usr/local/bin

