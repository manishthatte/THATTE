#!/usr/bin/env bash
set -euo pipefail

# Locate OVMF firmware (Debian/Ubuntu/Nix typical paths)
CANDIDATES_CODE=(
  "/usr/share/OVMF/OVMF_CODE.fd"
  "/usr/share/edk2/ovmf/OVMF_CODE.fd"
  "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd"
  # Nix
  "/run/current-system/sw/share/OVMF/OVMF_CODE.fd"
)

CANDIDATES_VARS=(
  "/usr/share/OVMF/OVMF_VARS.fd"
  "/usr/share/edk2/ovmf/OVMF_VARS.fd"
  "/usr/share/edk2-ovmf/x64/OVMF_VARS.fd"
  # Nix
  "/run/current-system/sw/share/OVMF/OVMF_VARS.fd"
)

find_first() {
  local -n arr=$1
  for p in "${arr[@]}"; do
    if [[ -f "$p" ]]; then
      echo "$p"
      return 0
    fi
  done
  return 1
}

OVMF_CODE="${OVMF_CODE:-$(find_first CANDIDATES_CODE || true)}"
OVMF_VARS_SRC="${OVMF_VARS:-$(find_first CANDIDATES_VARS || true)}"

if [[ -z "${OVMF_CODE}" || -z "${OVMF_VARS_SRC}" ]]; then
  echo "ERROR: Could not find OVMF firmware. Install 'ovmf' and set OVMF_CODE/OVMF_VARS env vars."
  exit 1
fi

BUILD_DIR="build"
ESP_IMG="${BUILD_DIR}/esp.img"
OVMF_VARS="${BUILD_DIR}/OVMF_VARS.fd"

if [[ ! -f "${ESP_IMG}" ]]; then
  echo "ERROR: ESP image ${ESP_IMG} not found. Run: make esp"
  exit 1
fi

mkdir -p "${BUILD_DIR}"
cp -f "${OVMF_VARS_SRC}" "${OVMF_VARS}"

# Prefer KVM if available; else TCG
ACCEL="tcg"
if [[ -w /dev/kvm ]]; then
  ACCEL="kvm:tcg"
fi

echo "[run] Using OVMF_CODE=${OVMF_CODE}"
echo "[run] Using OVMF_VARS=${OVMF_VARS}"

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
