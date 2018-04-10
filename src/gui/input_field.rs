use gui::wnd;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use gui::utils::FromWide;
use std::io;
use gui::context_stash::send_event;
use std::ffi::OsString;
use gui::INPUT_SEARCH_ID;
use gui::HASHMAP;
use gui::get_string;
use gui::INPUT_TEXT;

pub fn new(parent: HWND) -> io::Result<wnd::Wnd> {
    let input_params = wnd::WndParams::builder()
        .window_name(get_string(INPUT_TEXT))
        .class_name(get_string(WC_EDIT))
        .h_menu(INPUT_SEARCH_ID as HMENU)
        .style(WS_BORDER | WS_VISIBLE | ES_LEFT | WS_CHILD)
        .h_parent(parent)
        .build();
    wnd::Wnd::new(input_params)
}

pub unsafe fn on_change(_wnd: HWND, _w_param: WPARAM, l_param: LPARAM){
    let length = 1 + GetWindowTextLengthW(l_param as *mut _);
    let mut buffer = vec![0u16; length as usize];
    let read = 1 + GetWindowTextW(l_param as *mut _, buffer.as_mut_ptr(), length);
    assert_eq!(length, read);
    send_event(OsString::from_wide_null(&buffer));
    HASHMAP.lock().insert("hola", buffer);
}