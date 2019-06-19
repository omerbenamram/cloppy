use failure::Error;
use crate::gui::event::Event;
use crate::gui::Gui;
use crate::settings::Setting;
use crate::settings::setting_to_int;

pub fn restore_windows_position(_event: Event, gui: &mut Gui) -> Result<(), Error> {
    let wnd = gui.wnd();
    let settings = gui.settings();
    wnd.set_position(
        setting_to_int(Setting::WindowXPosition, settings),
        setting_to_int(Setting::WindowYPosition, settings),
        setting_to_int(Setting::WindowWidth, settings),
        setting_to_int(Setting::WindowHeight, settings),
        0,
    )
}
