#![no_std]
#![no_main]

use core::ptr;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::table::runtime::ResetType;
use uefi::CStr16;

#[entry]
fn efi_main(_image: Handle, mut st: SystemTable<Boot>) -> Status {
    let _ = st.stdout().reset(false);
    let _ = st.stdout().output_string(cstr16!("THATTE: UEFI hello stage starting...\r\n"));

    // GOP
    let bt = st.boot_services();
    let gop_ptr = match bt.locate_protocol::<GraphicsOutput>() {
        Ok(p) => p,
        Err(e) => { let _ = st.stdout().output_string(cstr16!("ERROR: GOP not available.\r\n")); return e.status(); }
    };
    let gop = unsafe { &mut *gop_ptr.get() };
    draw_scene(gop);

    let _ = st.stdout().output_string(cstr16!("THATTE: drew frame. Warm reboot in 5s...\r\n"));
    bt.stall(5_000_000);
    st.runtime_services().reset(ResetType::Warm, Status::SUCCESS, None)
}

fn draw_scene(gop: &mut GraphicsOutput) {
    let mode = gop.current_mode_info();
    let (w, h) = mode.resolution();
    let stride = mode.stride();
    let fmt = mode.pixel_format();
    let mut fb = gop.frame_buffer();
    let base = fb.as_mut_ptr();

    for y in 0..h {
        for x in 0..w {
            let fracx = (x as u32 * 255) / (w.max(1) as u32);
            let fracy = (y as u32 * 255) / (h.max(1) as u32);
            let r = (16 + (fracx / 3)) as u8;
            let g = (32 + (fracy / 4)) as u8;
            let b = 64u8;
            unsafe { put_pixel(base, x as usize, y as usize, stride as usize, fmt, r, g, b); }
        }
    }
    // Draw block letters "THATTE"
    let k = (h as usize / 16).max(8);
    let spacing = k / 2;
    let letter_w = 3 * k;
    let letter_h = 5 * k;
    let total_w = 6 * letter_w + 5 * spacing;
    let start_x = (w as usize / 2).saturating_sub(total_w / 2);
    let start_y = (h as usize / 2).saturating_sub(letter_h / 2);
    let fg = (220u8, 230u8, 245u8);
    let mut x = start_x;
    draw_t(base, stride as usize, fmt, x, start_y, k, fg); x += letter_w + spacing;
    draw_h(base, stride as usize, fmt, x, start_y, k, fg); x += letter_w + spacing;
    draw_a(base, stride as usize, fmt, x, start_y, k, fg); x += letter_w + spacing;
    draw_t(base, stride as usize, fmt, x, start_y, k, fg); x += letter_w + spacing;
    draw_t(base, stride as usize, fmt, x, start_y, k, fg); x += letter_w + spacing;
    draw_e(base, stride as usize, fmt, x, start_y, k, fg);
}

#[inline]
unsafe fn put_pixel(base: *mut u8, x: usize, y: usize, stride: usize, fmt: PixelFormat, r: u8, g: u8, b: u8) {
    let idx = ((y * stride) + x) * 4;
    match fmt {
        PixelFormat::Rgb => {
            ptr::write(base.add(idx + 0), r);
            ptr::write(base.add(idx + 1), g);
            ptr::write(base.add(idx + 2), b);
            ptr::write(base.add(idx + 3), 0);
        }
        _ => {
            ptr::write(base.add(idx + 0), b);
            ptr::write(base.add(idx + 1), g);
            ptr::write(base.add(idx + 2), r);
            ptr::write(base.add(idx + 3), 0);
        }
    }
}

fn draw_rect(base: *mut u8, stride: usize, fmt: PixelFormat, x: usize, y: usize, w: usize, h: usize, c: (u8,u8,u8)) {
    for yy in y..y+h { for xx in x..x+w { unsafe { put_pixel(base, xx, yy, stride, fmt, c.0, c.1, c.2); } } }
}
fn draw_t(base:*mut u8,stride:usize,fmt:PixelFormat,x:usize,y:usize,k:usize,c:(u8,u8,u8)){
    let w=3*k; let h=5*k; let t=k/3+1; draw_rect(base,stride,fmt,x,y,w,t,c); let vx=x+w/2-t/2; draw_rect(base,stride,fmt,vx,y,t,h,c);
}
fn draw_h(base:*mut u8,stride:usize,fmt:PixelFormat,x:usize,y:usize,k:usize,c:(u8,u8,u8)){
    let w=3*k; let h=5*k; let t=k/3+1; draw_rect(base,stride,fmt,x,y,t,h,c); draw_rect(base,stride,fmt,x+w-t,y,t,h,c); draw_rect(base,stride,fmt,x,y+2*k,w,t,c);
}
fn draw_a(base:*mut u8,stride:usize,fmt:PixelFormat,x:usize,y:usize,k:usize,c:(u8,u8,u8)){
    let w=3*k; let h=5*k; let t=k/3+1; draw_rect(base,stride,fmt,x,y+k,t,h-k,c); draw_rect(base,stride,fmt,x+w-t,y+k,t,h-k,c); draw_rect(base,stride,fmt,x+k/2,y,w-k,t,c); draw_rect(base,stride,fmt,x+t,y+2*k,w-2*t,t,c);
}
fn draw_e(base:*mut u8,stride:usize,fmt:PixelFormat,x:usize,y:usize,k:usize,c:(u8,u8,u8)){
    let w=3*k; let h=5*k; let t=k/3+1; draw_rect(base,stride,fmt,x,y,t,h,c); draw_rect(base,stride,fmt,x,y,w,t,c); draw_rect(base,stride,fmt,x,y+2*k,(w*4)/5,t,c); draw_rect(base,stride,fmt,x,y+h-t,w,t,c);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }
