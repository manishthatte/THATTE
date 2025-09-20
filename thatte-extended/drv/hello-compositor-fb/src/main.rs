//! Minimal fbdev compositor demo for DriverOS: paints a gradient to /dev/fb0.
//! Build (static, MUSL): cargo build --release --target x86_64-unknown-linux-musl
//! Copy to guest and run; or place in /opt/hello-compositor via driveros-make.sh.

use std::fs::File;
use std::io::{Read};
use std::os::fd::AsRawFd;
use std::ptr::null_mut;
use nix::sys::mman::{mmap, MapFlags, ProtFlags};
use libc::{ioctl, c_ulong};

const FBIOGET_VSCREENINFO: c_ulong = 0x4600;
const FBIOGET_FSCREENINFO: c_ulong = 0x4602;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct FbFixScreenInfo {
    id: [u8; 16],
    smem_start: u64,
    smem_len: u32,
    _type: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
    mmio_start: u64,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct FbVarScreenInfo {
    xres: u32, yres: u32,
    xres_virtual: u32, yres_virtual: u32,
    xoffset: u32, yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: FbBitField, green: FbBitField, blue: FbBitField, transp: FbBitField,
    nonstd: u32,
    activate: u32,
    height: u32, width: u32,
    accel_flags: u32,
    pixclock: u32, left_margin: u32, right_margin: u32, upper_margin: u32, lower_margin: u32, hsync_len: u32, vsync_len: u32, sync: u32, vmode: u32,
    rotate: u32,
    colorspace: u32,
    reserved: [u32; 4],
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct FbBitField { offset: u32, length: u32, msb_right: u32 }

fn main() {
    let path = "/dev/fb0";
    let f = match File::options().read(true).write(true).open(path) {
        Ok(v) => v,
        Err(e) => { eprintln!("open {} failed: {}", path, e); return; }
    };
    let fd = f.as_raw_fd();

    let mut fix = FbFixScreenInfo::default();
    let mut var = FbVarScreenInfo::default();

    unsafe {
        if ioctl(fd, FBIOGET_FSCREENINFO, &mut fix) != 0 {
            eprintln!("ioctl FBIOGET_FSCREENINFO failed (fbdev not available?)");
            return;
        }
        if ioctl(fd, FBIOGET_VSCREENINFO, &mut var) != 0 {
            eprintln!("ioctl FBIOGET_VSCREENINFO failed");
            return;
        }
    }

    let w = var.xres as usize;
    let h = var.yres as usize;
    let bpp = var.bits_per_pixel as usize;
    if bpp < 24 {
        eprintln!("Unsupported bpp={} (<24)", bpp);
        return;
    }
    let stride = fix.line_length as usize;
    let length = (stride * h) as usize;

    let map = unsafe {
        mmap(
            null_mut(),
            length,
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_SHARED,
            fd,
            0,
        )
    };
    if let Ok(ptr) = map {
        // Paint a simple gradient
        let base = ptr as *mut u8;
        for y in 0..h {
            for x in 0..w {
                let r = (16 + ((x * 255) / (w.max(1)))) as u8;
                let g = (32 + ((y * 255) / (h.max(1)))) as u8;
                let b = 64u8;
                let idx = y * stride + x * (bpp / 8);
                unsafe {
                    // Assume BGRX or RGBX; write both orders for safety (most fbdev are little endian BGRX)
                    *base.add(idx + 0) = b;
                    *base.add(idx + 1) = g;
                    *base.add(idx + 2) = r;
                    if bpp >= 32 { *base.add(idx + 3) = 0; }
                }
            }
        }
        eprintln!("Painted gradient {}x{} bpp{} stride{}", w, h, bpp, stride);
    } else {
        eprintln!("mmap failed");
    }
}
