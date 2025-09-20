# THATTE — Extended Starter (UEFI hello + VM manager + DriverOS scripts + hello-compositor)

This extends the minimal UEFI “hello” with:
- **Skeleton microkernel crate** (`mk/thatte-mk`) — builds as a freestanding library and runs unit tests on host via `std` feature.
- **VM manager** (`tools/vm-manager`) — a Rust CLI wrapper that launches a **DriverOS** VM under **QEMU/KVM** with virtio devices.
- **DriverOS provisioning scripts** (`scripts/driveros-*.sh`) — build a minimal Debian rootfs image, extract its kernel/initrd, and run it.
- **Hello Compositor** (`drv/hello-compositor-fb`) — a tiny Rust fbdev demo that paints a gradient on `/dev/fb0` inside the guest.

The original **UEFI stage** remains the Day‑0 pixel proof and is unaffected.

> This bundle is self-contained; the DriverOS image is created on your machine using Debian `debootstrap`. Internet is required for that step.
> Everything else builds offline.

---

## Prereqs (host: Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y build-essential qemu-system-x86 ovmf mtools dosfstools llvm lld clang make curl     debootstrap fdisk dosfstools kpartx e2fsprogs rsync sudo     musl-tools
# Rust (if not already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
~/.cargo/bin/rustup toolchain install nightly
~/.cargo/bin/rustup target add x86_64-unknown-uefi --toolchain nightly
~/.cargo/bin/rustup target add x86_64-unknown-linux-musl
```

Optional (Nix users): `nix develop` will provide pinned tools for the UEFI step; the DriverOS scripts still use host `debootstrap`.

---

## TL;DR commands

```bash
# 0) UEFI hello (as before)
make boot-uefi            # build BOOTX64.EFI
make esp && make run      # boot in QEMU/OVMF

# 1) Build hello-compositor (guest app; static MUSL binary)
make hello-compositor

# 2) Create DriverOS image (Debian stable, ~1–2GB)
sudo scripts/driveros-make.sh

# 3) Run DriverOS VM with virtio-gpu; it auto-launches hello-compositor if present
scripts/driveros-run.sh

# (Alternative) Use the Rust vm-manager wrapper
cargo run -p vm-manager -- --cfg configs/driveros.toml run
```

Expected: a QEMU window (virtio-gpu) with a gradient rendered by the guest on `/dev/fb0`.
If `/dev/fb0` is absent in your guest, switch QEMU video to `-vga std` or install `linux-image-amd64` with fbcon enabled.

---

## Repository layout additions

```
mk/thatte-mk/                 # skeleton microkernel library (no_std by default)
tools/vm-manager/             # Rust CLI wrapper around QEMU
drv/hello-compositor-fb/      # guest demo drawing via fbdev
configs/driveros.toml         # vm-manager config
scripts/driveros-make.sh      # build DriverOS disk image with debootstrap
scripts/driveros-run.sh       # run DriverOS with virtio-gpu
```

---

## Notes & limitations

- The microkernel crate is a scaffold; boot integration will come later.
- The vm-manager currently **execs QEMU**; later you can replace it with a KVM/rust‑vmm VMM.
- The compositor demo uses **fbdev** for simplicity; many configs provide `/dev/fb0` via simpledrm. If not, adjust QEMU args in `driveros-run.sh` (use `-vga std`) or install a DRM fb driver.
- All scripts are idempotent; you can re-run to update the image.
