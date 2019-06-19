use crate::gui::event::Event;
use crate::gui::Gui;
use failure::Error;

use crate::settings::Setting;
use std::collections::HashMap;

pub fn new_settings(event: Event, gui: &mut Gui) -> Result<(), Error> {
    let new_props: Box<HashMap<Setting, String>> = unsafe { Box::from_raw(event.w_param_mut()) };
    gui.set_settings(*new_props);
    println!("new properties");
    Ok(())
}
