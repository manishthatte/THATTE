#!/usr/bin/env bash
# Run DriverOS with virtio-gpu and KVM if available
set -euo pipefail

BUILD_DIR="build"
KERNEL="${KERNEL:-${BUILD_DIR}/vmlinuz}"
INITRD="${INITRD:-${BUILD_DIR}/initrd.img}"
DISK="${DISK:-${BUILD_DIR}/driveros.img}"

[[ -f "${KERNEL}" && -f "${INITRD}" && -f "${DISK}" ]] || {
  echo "Missing KERNEL/INITRD/DISK. Run: sudo scripts/driveros-make.sh"; exit 1; }

ACCEL="tcg"
[[ -w /dev/kvm ]] && ACCEL="kvm:tcg"

# virtio-gpu + std vga fallback for fbdev; you can switch to -device virtio-gpu-pci only.
VIDEO_OPTS="-vga std"

# 9p share for /hostshare
SHARE_DIR="${SHARE_DIR:-$(pwd)}"

exec qemu-system-x86_64 \
  -machine q35,accel=${ACCEL} \
  -cpu host \
  -m 2048 \
  -smp 4 \
  -kernel "${KERNEL}" \
  -initrd "${INITRD}" \
  -append "console=ttyS0 root=/dev/vda1 rw quiet" \
  -drive file="${DISK}",if=virtio,format=raw \
  ${VIDEO_OPTS} \
  -serial stdio \
  -fsdev local,id=fsdev0,path="${SHARE_DIR}",security_model=none \
  -device virtio-9p-pci,fsdev=fsdev0,mount_tag=hostshare \
  -device virtio-net-pci,netdev=n0 \
  -netdev user,id=n0,hostfwd=tcp::2222-:22
