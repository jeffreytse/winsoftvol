TARGET  := x86_64-pc-windows-gnu
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
GIT     := $(shell git rev-parse --short HEAD 2>/dev/null || echo unknown)
NAME    := sound-control-v$(VERSION)-$(GIT)

OUT     := target/$(TARGET)/release/sound-control.exe
DIST    := dist/$(NAME).exe

.PHONY: build release debug dist clean setup

build: release

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

# Install cross-compile toolchain (macOS only)
setup:
	rustup target add $(TARGET)
	brew install mingw-w64
