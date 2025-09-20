#!/usr/bin/env bash
set -euo pipefail

# Locate OVMF firmware
CANDIDATES_CODE=(
  "/usr/share/OVMF/OVMF_CODE.fd"
  "/usr/share/edk2/ovmf/OVMF_CODE.fd"
  "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd"
  "/run/current-system/sw/share/OVMF/OVMF_CODE.fd"
)
CANDIDATES_VARS=(
  "/usr/share/OVMF/OVMF_VARS.fd"
  "/usr/share/edk2/ovmf/OVMF_VARS.fd"
  "/usr/share/edk2-ovmf/x64/OVMF_VARS.fd"
  "/run/current-system/sw/share/OVMF/OVMF_VARS.fd"
)

find_first() {
  local -n arr=$1
  for p in "${arr[@]}"; do [[ -f "$p" ]] && { echo "$p"; return 0; }; done
  return 1
}

OVMF_CODE="${OVMF_CODE:-$(find_first CANDIDATES_CODE || true)}"
OVMF_VARS_SRC="${OVMF_VARS:-$(find_first CANDIDATES_VARS || true)}"
[[ -z "${OVMF_CODE}" || -z "${OVMF_VARS_SRC}" ]] && { echo "ERROR: OVMF not found"; exit 1; }

BUILD_DIR="build"
ESP_IMG="${BUILD_DIR}/esp.img"
OVMF_VARS="${BUILD_DIR}/OVMF_VARS.fd"
[[ -f "${ESP_IMG}" ]] || { echo "ERROR: ${ESP_IMG} not found. Run: make esp"; exit 1; }

cp -f "${OVMF_VARS_SRC}" "${OVMF_VARS}"

ACCEL="tcg"
[[ -w /dev/kvm ]] && ACCEL="kvm:tcg"

exec qemu-system-x86_64 \
  -machine q35,accel=${ACCEL} \
  -cpu host \
  -m 1024 \
  -serial stdio \
  -drive if=pflash,format=raw,readonly=on,file="${OVMF_CODE}" \
  -drive if=pflash,format=raw,file="${OVMF_VARS}" \
  -drive format=raw,file="${ESP_IMG}",if=virtio \
  -name "THATTE UEFI Hello" \
  -no-reboot
