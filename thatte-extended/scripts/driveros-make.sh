#!/usr/bin/env bash
# Build a minimal Debian-based DriverOS disk image and extract kernel/initrd.
# Requires: debootstrap, kpartx, e2fsprogs, rsync, sudo.
set -euo pipefail

IMG="build/driveros.img"
MNT="/mnt/driveros-build"
SIZE_GB="${SIZE_GB:-2}"
SUITE="${SUITE:-stable}"
ARCH="${ARCH:-amd64}"
MIRROR="${MIRROR:-http://deb.debian.org/debian}"
HOSTNAME="${HOSTNAME:-driveros}"
ROOTPWD="${ROOTPWD:-thatte}"
PKGS="${PKGS:-systemd-sysv,linux-image-amd64,openssh-server,net-tools,iproute2,ifupdown,ca-certificates,less,vim,udev,fbset}"

BUILD_DIR="build"
KERNEL_OUT="${BUILD_DIR}/vmlinuz"
INITRD_OUT="${BUILD_DIR}/initrd.img"

mkdir -p "${BUILD_DIR}"
sudo rm -f "${IMG}"
truncate -s "${SIZE_GB}G" "${IMG}"

# Partition: single root ext4
parted -s "${IMG}" mklabel msdos
parted -s "${IMG}" mkpart primary ext4 1MiB 100%

# Map partitions
LOOP=$(sudo losetup --show -f "${IMG}")
PART="${LOOP}p1"
sudo partprobe "${LOOP}"
# Some systems need kpartx:
if ! [ -e "${PART}" ]; then
  sudo kpartx -av "${LOOP}"
  PART="/dev/mapper/$(basename ${LOOP})p1"
fi

sudo mkfs.ext4 -F "${PART}"
sudo mkdir -p "${MNT}"
sudo mount "${PART}" "${MNT}"

echo "[*] debootstrap ${SUITE} -> ${MNT}"
sudo debootstrap --arch="${ARCH}" "${SUITE}" "${MNT}" "${MIRROR}"

echo "${HOSTNAME}" | sudo tee "${MNT}/etc/hostname" >/dev/null
echo "127.0.0.1 localhost
127.0.1.1 ${HOSTNAME}" | sudo tee "${MNT}/etc/hosts" >/dev/null

# Networking (DHCP on eth0)
echo "auto lo
iface lo inet loopback

allow-hotplug ens3
iface ens3 inet dhcp

allow-hotplug eth0
iface eth0 inet dhcp" | sudo tee "${MNT}/etc/network/interfaces" >/dev/null

# Locale/timezone minimal
echo "en_US.UTF-8 UTF-8" | sudo tee "${MNT}/etc/locale.gen" >/dev/null || true

# Sources list
echo "deb http://deb.debian.org/debian ${SUITE} main contrib non-free-firmware
deb http://security.debian.org/debian-security ${SUITE}-security main
deb http://deb.debian.org/debian ${SUITE}-updates main" | sudo tee "${MNT}/etc/apt/sources.list" >/dev/null

# Mount system dirs for chroot
for d in proc sys dev; do sudo mount --bind "/$d" "${MNT}/$d"; done

# Install packages
sudo chroot "${MNT}" bash -euxo pipefail -c "
apt-get update
DEBIAN_FRONTEND=noninteractive apt-get install -y ${PKGS}
echo 'root:${ROOTPWD}' | chpasswd
systemctl enable ssh
"

# Copy hello-compositor if built
if [[ -f build/hello-compositor ]]; then
  sudo install -D -m 0755 build/hello-compositor "${MNT}/opt/hello-compositor"
  # Create a simple autologin + run service on tty1
  sudo tee "${MNT}/etc/systemd/system/hello-compositor.service" >/dev/null <<'EOF'
[Unit]
Description=THATTE hello-compositor fbdev demo
After=multi-user.target

[Service]
Type=simple
TTYPath=/dev/tty1
StandardInput=tty
StandardOutput=tty
StandardError=journal
ExecStart=/opt/hello-compositor
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF
  sudo chroot "${MNT}" systemctl enable hello-compositor.service || true
fi

# Extract kernel & initrd to host
sudo bash -c "cp ${MNT}/boot/vmlinuz-* ${KERNEL_OUT}"
sudo bash -c "cp ${MNT}/boot/initrd.img-* ${INITRD_OUT}"

# fstab (root on /dev/vda1; virtio-blk)
echo "/dev/vda1 / ext4 defaults 0 1" | sudo tee -a "${MNT}/etc/fstab" >/dev/null

# Cleanup mounts
for d in dev sys proc; do sudo umount -lf "${MNT}/$d" || true; done
sudo umount -lf "${MNT}" || true

# Detach loops
if [[ "${PART}" == /dev/mapper/* ]]; then
  sudo kpartx -dv "${LOOP}" || true
fi
sudo losetup -d "${LOOP}"

echo "[OK] DriverOS image: ${IMG}"
echo "[OK] Kernel: ${KERNEL_OUT}"
echo "[OK] Initrd: ${INITRD_OUT}"
echo "[INFO] Default root password: ${ROOTPWD}"
