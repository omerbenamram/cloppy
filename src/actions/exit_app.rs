use crate::gui::event::Event;
use crate::gui::msg::Msg;
use crate::gui::Gui;
use failure::Error;
use winapi::um::winuser::MSG;

pub fn exit_app(_event: Event, _gui: &mut Gui) -> Result<(), Error> {
    MSG::post_quit(0);
    Ok(())
}
