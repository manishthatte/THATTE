#![cfg_attr(feature = "no_std", no_std)]
//! THATTE microkernel skeleton: capability ids and typed IPC model (stubs).
//!
//! This crate is currently a library that builds in `no_std` mode by default,
//! with some `std` tests for message encoding.

/// A 128-bit capability id (opaque).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Cap([u8; 16]);

impl Cap {
    pub const fn new(bytes: [u8; 16]) -> Self { Self(bytes) }
    pub fn nil() -> Self { Self([0u8; 16]) }
    pub fn bytes(&self) -> &[u8; 16] { &self.0 }
}

/// A tiny typed IPC message enum (to be codegen'd from IDL later).
#[repr(u16)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MsgType {
    Ping = 1,
    GetTime = 2,
    MapShared = 3,
}

/// IPC header (fixed size) followed by payload depending on `MsgType`.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IpcHeader {
    pub ty: u16,
    pub flags: u16,
    pub src: Cap,
    pub dst: Cap,
    pub len: u32,
}

impl core::fmt::Debug for IpcHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "IpcHeader {{ ty: {}, flags: {}, len: {} }}", self.ty, self.flags, self.len)
    }
}

/// Serialize a header to bytes (little-endian).
pub fn serialize_header(h: &IpcHeader) -> [u8; 40] {
    let mut out = [0u8; 40];
    out[0..2].copy_from_slice(&(h.ty).to_le_bytes());
    out[2..4].copy_from_slice(&(h.flags).to_le_bytes());
    out[4..20].copy_from_slice(h.src.bytes());
    out[20..36].copy_from_slice(h.dst.bytes());
    out[36..40].copy_from_slice(&(h.len).to_le_bytes());
    out
}

/// Parse a header from bytes (little-endian).
pub fn parse_header(buf: &[u8]) -> Option<IpcHeader> {
    if buf.len() < 40 { return None; }
    let ty = u16::from_le_bytes([buf[0], buf[1]]);
    let flags = u16::from_le_bytes([buf[2], buf[3]]);
    let mut src = [0u8; 16];
    src.copy_from_slice(&buf[4..20]);
    let mut dst = [0u8; 16];
    dst.copy_from_slice(&buf[20..36]);
    let len = u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]);
    Some(IpcHeader { ty, flags, src: Cap::new(src), dst: Cap::new(dst), len })
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn header_roundtrip(ty in 0u16..=u16::MAX, flags in 0u16..=u16::MAX, len in 0u32..=u32::MAX) {
            let h = IpcHeader { ty, flags, src: Cap::nil(), dst: Cap::nil(), len };
            let b = serialize_header(&h);
            let h2 = parse_header(&b).unwrap();
            prop_assert_eq!(h.ty, h2.ty);
            prop_assert_eq!(h.flags, h2.flags);
            prop_assert_eq!(h.len, h2.len);
        }
    }
}
