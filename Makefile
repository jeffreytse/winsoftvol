TARGET  := x86_64-pc-windows-gnu
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
GIT     := $(shell git rev-parse --short HEAD 2>/dev/null || echo unknown)
NAME    := winsoftvol-v$(VERSION)-$(GIT)

OUT     := target/$(TARGET)/release/winsoftvol.exe
DIST    := dist/$(NAME).exe

.PHONY: build release debug dist clean setup test lint fix

build: dist

release:
	cargo build --release --target $(TARGET)
	@echo "Binary : $(OUT)"
	@echo "Version: v$(VERSION) ($(GIT))"

debug:
	cargo build --target $(TARGET)

dist: release
	@mkdir -p dist
	cp $(OUT) $(DIST)
	@echo "Packaged: $(DIST)"

clean:
	cargo clean
	rm -rf dist

test:
	cargo test

lint:
	cargo fmt --check
	cargo clippy --target $(TARGET) -- -D warnings

fix:
	cargo fmt
	cargo clippy --fix --target $(TARGET) --allow-dirty --allow-staged -- -D warnings

# Install cross-compile toolchain (macOS only)
setup:
	rustup target add $(TARGET)
	brew install mingw-w64
