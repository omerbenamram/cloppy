use crate::gui::event::Event;
use crate::gui::Gui;
use failure::Error;

pub fn focus_on_input_field(_event: Event, gui: &mut Gui) -> Result<(), Error> {
    gui.input_search().wnd().set_focus()
}
