#!/usr/bin/env bash
set -euo pipefail

TARGET="x86_64-unknown-uefi"
BUILD_DIR="build"
EFI_BIN="${BUILD_DIR}/BOOTX64.EFI"
ESP_IMG="${BUILD_DIR}/esp.img"

echo "[build] Compiling (release)"
cargo +nightly build --target "${TARGET}" --release

mkdir -p "${BUILD_DIR}"
BIN=$(find "target/${TARGET}/release" -maxdepth 1 -type f \( -name '*.efi' -o -name '*.dll' -o -name '*.so' \) | head -n1 || true)
if [[ -z "${BIN}" ]]; then
  echo "ERROR: EFI artifact not found."
  exit 1
fi
cp -f "${BIN}" "${EFI_BIN}"

echo "[esp] Building FAT32 ESP"
dd if=/dev/zero of="${ESP_IMG}" bs=1M count=64 status=none
mkfs.vfat -F 32 "${ESP_IMG}" >/dev/null
mmd -i "${ESP_IMG}" ::/EFI ::/EFI/BOOT
mcopy -i "${ESP_IMG}" "${EFI_BIN}" ::/EFI/BOOT/BOOTX64.EFI
echo "[done] ${ESP_IMG} ready. Run: scripts/run-qemu.sh"
