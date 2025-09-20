# THATTE — Starter Repo (UEFI “hello kernel” + QEMU)

This is a *minimal, reproducible* boot stage for **THATTE** that boots as a UEFI application,
switches to graphics mode, paints a gradient background, and draws block letters spelling **THATTE**.
It provides a deterministic build (via Nix shell) and pragmatic Debian instructions, plus simple `make` targets.

> Goal for Day 0: get **pixels on screen** under QEMU on any Debian‑like workstation, and have a clean
> structure to evolve into the microkernel + DriverOS architecture.

---

## Quick start (Debian/Ubuntu, no Nix)

1. **Install toolchain and firmware** (one-time):

```bash
sudo apt update
sudo apt install -y build-essential qemu-system-x86 ovmf mtools dosfstools llvm lld clang make curl
# Install Rust (if you don't have it yet); this installs rustup into ~/.cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# Add the UEFI target + nightly (we use nightly for target support)
~/.cargo/bin/rustup toolchain install nightly
~/.cargo/bin/rustup target add x86_64-unknown-uefi --toolchain nightly
```

2. **Build the UEFI stage and ESP image**:

```bash
make build
make esp
```

3. **Run in QEMU (UEFI/OVMF)**:

```bash
make run
```

If QEMU can't find OVMF firmware automatically, edit `scripts/run-qemu.sh` and set the `OVMF_CODE`/`OVMF_VARS` paths explicitly.

4. **Clean**:

```bash
make clean
```

---

## Quick start (Nix Flakes)

If you have **Nix** with flakes enabled:

```bash
nix develop
make build
make esp
make run
```

The dev shell provides `rustc`/`cargo` (nightly with the UEFI target), QEMU, OVMF, `mtools`, `dosfstools`, `lld`, and `clang` pinned by Nixpkgs.

---

## What you get today

- `boot/thatte-boot-efi/` — Rust UEFI program (no_std, no_main) that:
  - Initializes via the UEFI entry point
  - Locates the **Graphics Output Protocol** (GOP)
  - Paints a gradient background
  - Draws “THATTE” using filled rectangles
  - Reboots after 5 seconds (so you see a full boot cycle in CI later)

- `scripts/` — build & run helpers:
  - `build.sh` — builds the EFI binary and prepares a FAT32 ESP image
  - `run-qemu.sh` — boots the image with **OVMF** (UEFI) in QEMU

- `Makefile` — convenience targets (`build`, `esp`, `run`, `clean`).

- `flake.nix` — reproducible dev environment (Rust nightly + UEFI target).

This is intentionally *small*. From here, you can pivot to the microkernel in `mk/`,
introduce KVM and DriverOS, and add a compositor service instead of firmware drawing.

---

## Phased tests

1. **Smoke**: `make run` shows a window with a blue/teal gradient and the word **THATTE** centered.
2. **Reboot**: after ~5 seconds, the VM should reset (warm reboot) and show the same screen again.
3. **Console**: QEMU’s serial console (`-serial stdio`) shows the OVMF boot log; no critical errors expected.

---

## Troubleshooting

- **OVMF not found**: Install the package and check typical paths:

  - `/usr/share/OVMF/OVMF_CODE.fd` and `/usr/share/OVMF/OVMF_VARS.fd`
  - `/usr/share/edk2/ovmf/OVMF_CODE.fd` and `/usr/share/edk2/ovmf/OVMF_VARS.fd`
  - `/usr/share/edk2-ovmf/x64/OVMF_CODE.fd` and `/usr/share/edk2-ovmf/x64/OVMF_VARS.fd`

  Update the variables at the top of `scripts/run-qemu.sh` if necessary.

- **Permission denied creating ESP image**: The build writes to `./build/`. Ensure you have write permission to the repo directory.

- **Slow gradient on very high resolutions**: UEFI GOP modes differ. If the default is 4K and renders slowly under TCG,
  press `Esc` in the QEMU OVMF splash to select a lower resolution video mode, or increase QEMU RAM/CPU.

---

## Next steps toward THATTE Core

- Replace gradient drawing with a **compositor process** (your future Thayland) after exiting boot services.
- Add a basic **capability table** and a user task loader stub.
- Integrate **KVM + rust-vmm** to boot **DriverOS** in a side VM for GPU drivers, bridged via virtio-gpu.

---

## License

Apache-2.0 (for this starter). See `LICENSE`.
