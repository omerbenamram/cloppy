use failure::Error;
use crate::gui::event::Event;
use crate::gui::Gui;
use crate::gui::msg::Msg;
use winapi::um::winuser::MSG;

pub fn exit_app(_event: Event, _gui: &mut Gui) -> Result<(), Error> {
    MSG::post_quit(0);
    Ok(())
}
