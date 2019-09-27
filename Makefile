
.PHONY: compile install

compile:
	cargo build --release

install: compile
	cp target/release/termassist /usr/local/bin

