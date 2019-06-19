use crate::gui::event::Event;
use crate::gui::Gui;
use crate::plugin::State;
use failure::Error;

pub fn new_plugin_state(event: Event, gui: &mut Gui) -> Result<(), Error> {
    let new_state: Box<State> = unsafe { Box::from_raw(event.w_param_mut()) };
    gui.status_bar_mut().update(&new_state)?;
    gui.item_list_mut().update(&new_state)?;
    gui.dispatcher_mut().set_state(new_state);
    Ok(())
}
