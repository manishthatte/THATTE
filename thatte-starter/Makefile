SHELL := /bin/bash

TARGET := x86_64-unknown-uefi
BUILD_DIR := build
ESP_IMG := $(BUILD_DIR)/esp.img
EFI_BIN := $(BUILD_DIR)/BOOTX64.EFI

.PHONY: all setup build esp run clean

all: build esp

setup:
	@echo "[setup] Installing prerequisites (Debian/Ubuntu)"
	@sudo apt update && sudo apt install -y build-essential qemu-system-x86 ovmf mtools dosfstools llvm lld clang make curl || true
	@~/.cargo/bin/rustup toolchain install nightly || true
	@~/.cargo/bin/rustup target add $(TARGET) --toolchain nightly || true

build:
	@echo "[build] Compiling UEFI stage (release)"
	cargo +nightly build --target $(TARGET) --release
	@mkdir -p $(BUILD_DIR)
	@echo "[build] Locating EFI artifact"
	@BIN=$$(find target/$(TARGET)/release -maxdepth 1 -type f \( -name '*.efi' -o -name '*.dll' -o -name '*.so' \) | head -n1); \
	if [[ -z "$$BIN" ]]; then echo "ERROR: EFI artifact not found. Check the build output."; exit 1; fi; \
	cp "$$BIN" $(EFI_BIN).tmp && mv $(EFI_BIN).tmp $(EFI_BIN)
	@echo "[build] Output -> $(EFI_BIN)"

esp: build
	@echo "[esp] Creating FAT32 ESP image"
	@rm -rf $(BUILD_DIR)/esp
	@mkdir -p $(BUILD_DIR)
	@dd if=/dev/zero of=$(ESP_IMG) bs=1M count=64 status=none
	@mkfs.vfat -F 32 $(ESP_IMG) >/dev/null
	@mmd -i $(ESP_IMG) ::/EFI ::/EFI/BOOT
	@mcopy -i $(ESP_IMG) $(EFI_BIN) ::/EFI/BOOT/BOOTX64.EFI
	@echo "[esp] ESP image ready -> $(ESP_IMG)"

run: esp
	@bash scripts/run-qemu.sh

clean:
	@echo "[clean] Removing build artifacts"
	@cargo clean
	@rm -rf $(BUILD_DIR)
