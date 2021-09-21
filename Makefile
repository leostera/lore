LORE_EXE=lore

.PHONY: build
build:
	cargo build

.PHONY: manual
manual:
	mdbook build --dest-dir ../docs ./manual

.PHONY: test
test:
	cargo test \
		--no-fail-fast \
		--tests

.PHONY: release.win
release.win: LORE_EXE=lore.exe
release.win: release

.PHONY: release
release:
	cargo build --release
	cargo strip
	tar czf release.tar.gz -C ./target/release/ $(LORE_EXE)

.PHONY: install
install:
	cargo install --path ./lib/lore-bin

.PHONY: setup
setup:
	cargo install cargo-strip cargo-insta mdbook

.PHONY: clean
clean:
	cargo clean

.PHONY: fmt
fmt:
	cargo fmt
