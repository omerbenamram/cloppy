pub use self::api_calls::*;
pub use self::structs::*;
use byteorder::{ByteOrder, LittleEndian};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

mod api_calls;
mod structs;

pub fn windows_string(input: &[u8]) -> String {
    let mut x: Vec<u16> = vec![];
    for c in input.chunks(2) {
        x.push(LittleEndian::read_u16(c));
    }
    OsString::from_wide(&x[..]).into_string().unwrap()
}
