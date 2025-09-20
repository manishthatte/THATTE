SHELL := /bin/bash

UEFI_TARGET := x86_64-unknown-uefi
BUILD_DIR := build
ESP_IMG := $(BUILD_DIR)/esp.img
EFI_BIN := $(BUILD_DIR)/BOOTX64.EFI

.PHONY: all boot-uefi esp run clean hello-compositor

all: boot-uefi esp

boot-uefi:
	@echo "[build] Compiling UEFI boot (release)"
	cargo +nightly build -p thatte-boot-efi --target $(UEFI_TARGET) --release
	@mkdir -p $(BUILD_DIR)
	@BIN=$$(find target/$(UEFI_TARGET)/release -maxdepth 1 -type f \( -name '*.efi' -o -name '*.dll' -o -name '*.so' \) | head -n1); \
	if [[ -z "$$BIN" ]]; then echo "ERROR: EFI artifact not found. Check the build output."; exit 1; fi; \
	cp "$$BIN" $(EFI_BIN).tmp && mv $(EFI_BIN).tmp $(EFI_BIN)
	@echo "[build] Output -> $(EFI_BIN)"

esp: boot-uefi
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

hello-compositor:
	@echo "[build] hello-compositor (static musl)"
	rustup target add x86_64-unknown-linux-musl >/dev/null 2>&1 || true
	cargo build -p hello-compositor-fb --release --target x86_64-unknown-linux-musl
	@mkdir -p $(BUILD_DIR)
	@cp target/x86_64-unknown-linux-musl/release/hello-compositor-fb $(BUILD_DIR)/hello-compositor

clean:
	@echo "[clean] Removing build artifacts"
	@cargo clean
	@rm -rf $(BUILD_DIR)
