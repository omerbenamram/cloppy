#![allow(dead_code)]
#[macro_use]
extern crate bitflags;
extern crate conv;
#[macro_use]
extern crate typed_builder;
extern crate winapi;

use gui::msg::Msg;
use gui::paint;
use gui::tray_icon;
use gui::utils;
use gui::utils::Location;
use gui::utils::ToWide;
use gui::wnd;
use gui::wnd_class;
use std::io;
use std::mem;
use std::ptr;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::minwindef::{
    BOOL,
    HIWORD,
    LOWORD,
    TRUE,
};
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::{
    HFONT,
    HWND,
};
use winapi::um::commctrl::{
    STATUSCLASSNAME,
    WC_EDIT,
};
use winapi::um::shellapi::{
    NIN_KEYSELECT,
    NIN_SELECT,
};
use winapi::um::wingdi::{
    CreateFontIndirectW,
    LOGFONTW,
};
use winapi::um::winuser::{
    CreateMenu,
    DefWindowProcW,
    EnumChildWindows,
    FindWindowExW,
    InsertMenuItemW,
    LoadAcceleratorsW,
    MAKEINTRESOURCEW,
    MENUITEMINFOW,
    MFS_ENABLED,
    MFT_STRING,
    MIIM_DATA,
    MIIM_FTYPE,
    MIIM_ID,
    MIIM_STRING,
    SendMessageW,
    SetMenu,
    SetWindowPos,
    WM_APP,
    WM_COMMAND,
    WM_CREATE,
    WM_DESTROY,
    WM_KEYDOWN,
    WM_LBUTTONDBLCLK,
    WM_LBUTTONUP,
    WM_PAINT,
    WM_SETFONT,
    WM_SIZE,
    EM_SETSEL,
};
use winapi::um::winuser::{
    MSG, NONCLIENTMETRICSW,
    SPI_GETNONCLIENTMETRICS,
    SWP_NOMOVE,
    SystemParametersInfoW,
    WM_QUIT,
};

mod gui;
mod resources;

use resources::constants::*;

const STATUS_BAR: u32 = 123;
const MAIN_WND_CLASS: &str = "hello";
const MAIN_WND_NAME: &str = "hello";
pub const WM_SYSTRAYICON: u32 = WM_APP + 1;
const INPUT_MARGIN: i32 = 5;

fn main() {
    match try_main() {
        Ok(code) => ::std::process::exit(code),
        Err(err) => {
            let msg = format!("Error: {}", err);
            panic!(msg);
        }
    }
}

fn default_font() -> Result<HFONT, io::Error> {
    unsafe {
        let mut metrics = mem::zeroed::<NONCLIENTMETRICSW>();
        let size = mem::size_of::<NONCLIENTMETRICSW>() as u32;
        metrics.cbSize = size;
        let font = match SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            size,
            &mut metrics as *mut _ as *mut _,
            0)
            {
                v if v == 0 => utils::last_error(),
                _ => Ok(metrics.lfMessageFont),
            }?;
        match CreateFontIndirectW(&font) {
            v if v.is_null() => utils::other_error("CreateFontIndirectW failed"),
            v => Ok(v)
        }
    }
}

fn try_main() -> io::Result<i32> {
    wnd_class::WndClass::init_commctrl()?;
    let class = wnd_class::WndClass::new(MAIN_WND_CLASS, wnd_proc)?;
    let accel = match unsafe { LoadAcceleratorsW(class.1, MAKEINTRESOURCEW(101)) } {
        v if v.is_null() => utils::other_error("LoadAccelerator failed"),
        v => Ok(v)
    }.unwrap();

    let params = wnd::WndParams::builder()
        .window_name(MAIN_WND_NAME)
        .class_name(class.0)
        .instance(class.1)
        .style(wnd::WndStyle::WS_OVERLAPPEDWINDOW)
        .build();
    let wnd = wnd::Wnd::new(params)?;
    main_menu(wnd.hwnd)?;
    let status_bar_params = wnd::WndParams::builder()
        .window_name("mystatusbar")
        .class_name(STATUSCLASSNAME.to_wide_null().as_ptr() as LPCWSTR)
        .instance(class.1)
        .h_parent(wnd.hwnd)
        .style(wnd::WndStyle::WS_VISIBLE | wnd::WndStyle::SBARS_SIZEGRIP | wnd::WndStyle::WS_CHILD)
        .build();
    wnd::Wnd::new(status_bar_params)?;
    let input_params = wnd::WndParams::builder()
        .window_name("myinputtext")
        .class_name(WC_EDIT.to_wide_null().as_ptr() as LPCWSTR)
        .instance(class.1)
        .style(wnd::WndStyle::WS_VISIBLE | wnd::WndStyle::WS_BORDER | wnd::WndStyle::ES_LEFT | wnd::WndStyle::WS_CHILD)
        .h_parent(wnd.hwnd)
        .location(Location { x: INPUT_MARGIN, y: INPUT_MARGIN })
        .build();
    let input = wnd::Wnd::new(input_params)?;
    wnd.show(winapi::um::winuser::SW_SHOWDEFAULT);
    wnd.update()?;
    unsafe { EnumChildWindows(wnd.hwnd, Some(font_proc), default_font().unwrap() as LPARAM); }
    let mut icon = tray_icon::TrayIcon::new(&wnd);
    icon.set_visible()?;
    loop {
        match MSG::get(None).unwrap() {
            MSG { message: WM_QUIT, wParam: code, .. } => {
                return Ok(code as i32);
            }
            mut msg => {
                if !msg.translate_accel(wnd.hwnd, accel) {
                    msg.translate();
                    msg.dispatch();
                }
            }
        }
    }
}

fn generate_layout(main: HWND, input: HWND) {}

fn main_menu(wnd: HWND) -> io::Result<()> {
    unsafe {
        let result = match CreateMenu() {
            v if v.is_null() => utils::last_error(),
            v => Ok(v)
        };
        let menu = result?;
        let x: MENUITEMINFOW = MENUITEMINFOW {
            cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_ID | MIIM_STRING | MIIM_DATA | MIIM_FTYPE,
            fType: MFT_STRING,
            fState: MFS_ENABLED,
            wID: 1,
            hSubMenu: ptr::null_mut(),
            hbmpChecked: ptr::null_mut(),
            hbmpUnchecked: ptr::null_mut(),
            dwItemData: 0,
            dwTypeData: "&File".to_wide_null().as_mut_ptr(),
            cch: "File".len() as u32,
            hbmpItem: ptr::null_mut(),
        };
        let result = match InsertMenuItemW(menu, 0, 1, &x) {
            0 => utils::last_error(),
            _ => Ok(())
        };
        let _ = result?;
        match SetMenu(wnd, menu) {
            0 => utils::last_error(),
            _ => Ok(())
        }
    }
}

unsafe extern "system" fn font_proc(wnd: HWND, font: LPARAM) -> BOOL {
    SendMessageW(wnd, WM_SETFONT, font as WPARAM, TRUE as LPARAM);
    TRUE
}

unsafe extern "system" fn wnd_proc(wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match message {
        WM_DESTROY => {
            MSG::post_quit(0);
            0
        }
        WM_SIZE => {
            let new_width = LOWORD(l_param as u32);
            let new_height = HIWORD(l_param as u32);
            let input_text = FindWindowExW(wnd, ptr::null_mut(), WC_EDIT.to_wide_null().as_ptr() as LPCWSTR, ptr::null_mut());
//            SendMessageW(input_text, WM_SIZE, 0, (new_height as LPARAM) << 16);
            SetWindowPos(input_text, ptr::null_mut(), 0, 0, new_width as i32 - 2 * INPUT_MARGIN, 20, SWP_NOMOVE);
            let status_bar = FindWindowExW(wnd, ptr::null_mut(), STATUSCLASSNAME.to_wide_null().as_ptr() as LPCWSTR, ptr::null_mut());
            SendMessageW(status_bar, WM_SIZE, 0, 0);
            DefWindowProcW(wnd, message, w_param, l_param)
        }
        WM_SYSTRAYICON => {
            match l_param as u32 {
                NIN_KEYSELECT | NIN_SELECT | WM_LBUTTONUP => {
                    println!("selected");
                }
                WM_LBUTTONDBLCLK => {
                    println!("double click");
                }
                _ => {}
            };
            0
        }
//        WM_SYSCOMMAND => {
//            println!("{:?}-{:?}-{:?}", message, w_param & 0xFFF0, l_param);
//            0
//
//        }
        WM_COMMAND => {
            match LOWORD(w_param as u32) as u32 {
                ID_SELECT_ALL => {
                    let input_text = FindWindowExW(wnd, ptr::null_mut(), WC_EDIT.to_wide_null().as_ptr() as LPCWSTR, ptr::null_mut());
                    SendMessageW(input_text, EM_SETSEL as u32, 0, -1);
                }
                _ => {}
            }
            DefWindowProcW(wnd, message, w_param, l_param)
        }
//        WM_KEYDOWN => {
//
//        }
        WM_PAINT => {
            let paint = paint::WindowPaint::new(wnd).unwrap();
            paint.text("Hello world", utils::Location { x: 10, y: 10 }).unwrap();
            DefWindowProcW(wnd, message, w_param, l_param)
        }
//        WM_RBUTTONUP => {
//            println!("holaa");
//            0
//        }
        message => DefWindowProcW(wnd, message, w_param, l_param),
    }
}
