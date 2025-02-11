use crate::actions::Action;
use crate::actions::SimpleAction;
use crate::gui::event::Event;
use crate::gui::Wnd;
use failure::Error;
use failure::ResultExt;
use num::FromPrimitive;
use slog::Logger;
use std::io;
use winapi::shared::minwindef::WPARAM;
use winapi::um::winuser::RegisterHotKey;
use winapi::um::winuser::MOD_ALT;
use winapi::um::winuser::MOD_NOREPEAT;
use winapi::um::winuser::MOD_WIN;

const N_KEY: u32 = 0x4E;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum Shortcut {
    RestoreWindow,
}
}

impl Shortcut {
    pub fn from_wparam(w_param: WPARAM) -> Option<Shortcut> {
        Shortcut::from_i32(w_param as i32)
    }
}

pub fn on_hotkey_event(logger: &Logger, event: Event) -> Action {
    let id = event.w_param() as i32;
    match Shortcut::from_i32(id) {
        None => {
            warn!(logger, "unknown shortcut"; "id" => id, "type" => "shortcut");
            SimpleAction::DoNothing.into()
        }
        Some(shortcut) => {
            info!(logger, "handling shortcut"; "id" => ?shortcut, "type" => "shortcut");
            Action::from(shortcut)
        }
    }
}

pub fn register_global_files(wnd: &Wnd) -> Result<(), Error> {
    unsafe {
        match RegisterHotKey(
            wnd.hwnd,
            Shortcut::RestoreWindow as i32,
            (MOD_WIN | MOD_ALT | MOD_NOREPEAT) as u32,
            N_KEY,
        ) {
            v if v == 0 => Err(io::Error::last_os_error()).with_context(|e| {
                let key = "WIN + ALT + N";
                format!("Could not register key {}: {}", key, e)
            })?,
            _ => Ok(()),
        }
    }
}
