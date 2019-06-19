use failure::Error;
use crate::gui::event::Event;
use crate::gui::Gui;
use winapi::um::winuser::SW_HIDE;

pub fn minimize_to_tray(_event: Event, gui: &mut Gui) -> Result<(), Error> {
    gui.wnd().show(SW_HIDE)
}