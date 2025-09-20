#![no_std]
#![no_main]

use core::cmp::max;
use core::ptr;

use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::table::runtime::ResetType;
use uefi::CStr16;

#[entry]
fn efi_main(_image: Handle, mut st: SystemTable<Boot>) -> Status {
    // Try to print something early to the text console.
    let _ = st.stdout().reset(false);
    let hello: &CStr16 = cstr16!("THATTE: UEFI hello stage starting...\r\n");
    let _ = st.stdout().output_string(hello);

    // Locate GOP (Graphics Output Protocol)
    let bt = st.boot_services();
    let mut gop = match bt
        .get_handle_for_protocol::<GraphicsOutput>()
        .and_then(|handle| bt.open_protocol_exclusive::<GraphicsOutput>(handle))
    {
        Ok(gop) => gop,
        Err(e) => {
            let _ = st
                .stdout()
                .output_string(cstr16!("ERROR: GOP not available.\r\n"));
            return e.status();
        }
    };

    // Draw scene
    draw_scene(&mut gop);

    let _ = st.stdout().output_string(cstr16!(
        "THATTE: drew frame. Warm reboot in 5 seconds...\r\n"
    ));

    // Wait a bit so the user can see the screen
    bt.stall(5_000_000); // microseconds

    // Reset (demonstrates control back to firmware)
    st.runtime_services()
        .reset(ResetType::WARM, Status::SUCCESS, None)
}

fn draw_scene(gop: &mut GraphicsOutput) {
    let mode = gop.current_mode_info();
    let (width, height) = mode.resolution();
    let stride = mode.stride();
    let format = mode.pixel_format();

    let mut fb = gop.frame_buffer();
    let base = fb.as_mut_ptr();

    // Background gradient
    for y in 0..height {
        for x in 0..width {
            let fracx = (x as u32 * 255) / max(1, width as u32);
            let fracy = (y as u32 * 255) / max(1, height as u32);
            let r = (16 + (fracx / 3)) as u8;
            let g = (32 + (fracy / 4)) as u8;
            let b = 64u8;
            unsafe {
                put_pixel(base, x as usize, y as usize, stride as usize, format, r, g, b);
            }
        }
    }

    // Draw the word "THATTE" centered
    let k = max(8usize, height as usize / 16);
    let spacing = k / 2;
    let letter_w = 3 * k;
    let letter_h = 5 * k;

    let total_w = 6 * letter_w + 5 * spacing;
    let start_x = (width as usize / 2).saturating_sub(total_w / 2);
    let start_y = (height as usize / 2).saturating_sub(letter_h / 2);

    let fg = (220u8, 230u8, 245u8);

    let mut x = start_x;
    draw_letter_t(base, stride as usize, format, x, start_y, k, fg);
    x += letter_w + spacing;
    draw_letter_h(base, stride as usize, format, x, start_y, k, fg);
    x += letter_w + spacing;
    draw_letter_a(base, stride as usize, format, x, start_y, k, fg);
    x += letter_w + spacing;
    draw_letter_t(base, stride as usize, format, x, start_y, k, fg);
    x += letter_w + spacing;
    draw_letter_t(base, stride as usize, format, x, start_y, k, fg);
    x += letter_w + spacing;
    draw_letter_e(base, stride as usize, format, x, start_y, k, fg);
}

#[inline]
unsafe fn put_pixel(
    base: *mut u8,
    x: usize,
    y: usize,
    stride: usize,
    fmt: PixelFormat,
    r: u8,
    g: u8,
    b: u8,
) {
    let idx = ((y * stride) + x) * 4;
    match fmt {
        PixelFormat::Rgb => {
            ptr::write(base.add(idx + 0), r);
            ptr::write(base.add(idx + 1), g);
            ptr::write(base.add(idx + 2), b);
            ptr::write(base.add(idx + 3), 0);
        }
        PixelFormat::Bgr | PixelFormat::Bitmask | PixelFormat::BltOnly => {
            // Treat others as BGRx for our purposes
            ptr::write(base.add(idx + 0), b);
            ptr::write(base.add(idx + 1), g);
            ptr::write(base.add(idx + 2), r);
            ptr::write(base.add(idx + 3), 0);
        }
    }
}

fn draw_rect(
    base: *mut u8,
    stride: usize,
    fmt: PixelFormat,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: (u8, u8, u8),
) {
    let (r, g, b) = color;
    for yy in y..y + h {
        for xx in x..x + w {
            unsafe { put_pixel(base, xx, yy, stride, fmt, r, g, b) }
        }
    }
}

fn draw_letter_t(
    base: *mut u8,
    stride: usize,
    fmt: PixelFormat,
    x: usize,
    y: usize,
    k: usize,
    color: (u8, u8, u8),
) {
    let w = 3 * k;
    let h = 5 * k;
    let t = k / 3 + 1;
    draw_rect(base, stride, fmt, x, y, w, t, color);
    let vx = x + w / 2 - t / 2;
    draw_rect(base, stride, fmt, vx, y, t, h, color);
}

fn draw_letter_h(
    base: *mut u8,
    stride: usize,
    fmt: PixelFormat,
    x: usize,
    y: usize,
    k: usize,
    color: (u8, u8, u8),
) {
    let w = 3 * k;
    let h = 5 * k;
    let t = k / 3 + 1;
    // pillars
    draw_rect(base, stride, fmt, x, y, t, h, color);
    draw_rect(base, stride, fmt, x + w - t, y, t, h, color);
    // crossbar
    draw_rect(base, stride, fmt, x, y + 2 * k, w, t, color);
}

fn draw_letter_a(
    base: *mut u8,
    stride: usize,
    fmt: PixelFormat,
    x: usize,
    y: usize,
    k: usize,
    color: (u8, u8, u8),
) {
    let w = 3 * k;
    let h = 5 * k;
    let t = k / 3 + 1;
    draw_rect(base, stride, fmt, x, y + k, t, h - k, color);
    draw_rect(base, stride, fmt, x + w - t, y + k, t, h - k, color);
    draw_rect(base, stride, fmt, x + k / 2, y, w - k, t, color);
    draw_rect(base, stride, fmt, x + t, y + 2 * k, w - 2 * t, t, color);
}

fn draw_letter_e(
    base: *mut u8,
    stride: usize,
    fmt: PixelFormat,
    x: usize,
    y: usize,
    k: usize,
    color: (u8, u8, u8),
) {
    let w = 3 * k;
    let h = 5 * k;
    let t = k / 3 + 1;
    draw_rect(base, stride, fmt, x, y, t, h, color);
    draw_rect(base, stride, fmt, x, y, w, t, color);
    draw_rect(base, stride, fmt, x, y + 2 * k, (w * 4) / 5, t, color);
    draw_rect(base, stride, fmt, x, y + h - t, w, t, color);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}