use failure::Error;
use crate::gui::event::Event;
use crate::gui::Gui;

pub fn focus_on_input_field(_event: Event, gui: &mut Gui) -> Result<(), Error> {
    gui.input_search().wnd().set_focus()
}
