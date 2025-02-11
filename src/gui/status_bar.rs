use crate::errors::MyErrorKind::WindowsError;
use crate::gui::get_string;
use crate::gui::set_string;
use crate::gui::wnd::Wnd;
use crate::gui::wnd::WndParams;
use crate::gui::STATUS_BAR;
use crate::gui::STATUS_BAR_CONTENT;
use crate::gui::STATUS_BAR_ID;
use crate::plugin::State;
use failure::Error;
use failure::ResultExt;
use std::io;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;

pub fn new(parent: Wnd, instance: Option<HINSTANCE>) -> Result<Wnd, Error> {
    let status_bar_params = WndParams::builder()
        .instance(instance)
        .window_name(get_string(STATUS_BAR))
        .h_menu(STATUS_BAR_ID as HMENU)
        .class_name(get_string(STATUSCLASSNAME))
        .h_parent(parent.hwnd)
        .style(WS_VISIBLE | SBARS_SIZEGRIP | WS_CHILD)
        .build();
    Ok(Wnd::new(status_bar_params).context(WindowsError("Failed to create wnd status_bar"))?)
}

pub struct StatusBar {
    wnd: Wnd,
}

impl StatusBar {
    pub fn new(wnd: Wnd) -> StatusBar {
        StatusBar { wnd }
    }

    pub fn wnd(&self) -> &Wnd {
        &self.wnd
    }

    pub fn update(&self, state: &State) -> Result<(), Error> {
        let msg = state.count().to_string() + " objects found";
        set_string(STATUS_BAR_CONTENT, msg.to_string());
        let w_param = (SB_SIMPLEID & (0 << 8)) as WPARAM;
        match self.wnd.send_message(
            SB_SETTEXTW,
            w_param,
            get_string(STATUS_BAR_CONTENT) as LPARAM,
        ) {
            0 => Err(io::Error::last_os_error()).context(WindowsError("SB_SETTEXTW failed"))?,
            _ => Ok(()),
        }
    }
}
