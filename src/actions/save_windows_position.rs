use crate::dispatcher::UiAsyncMessage;
use crate::gui::event::Event;
use crate::gui::Gui;
use crate::settings::Setting;
use failure::Error;
use std::collections::HashMap;

pub fn save_windows_position(_event: Event, gui: &mut Gui) -> Result<(), Error> {
    gui.wnd().window_rect().map(|rect| {
        let mut properties = HashMap::new();
        properties.insert(Setting::WindowXPosition, rect.left.to_string());
        properties.insert(Setting::WindowYPosition, rect.top.to_string());
        properties.insert(Setting::WindowHeight, (rect.bottom - rect.top).to_string());
        properties.insert(Setting::WindowWidth, (rect.right - rect.left).to_string());
        gui.dispatcher()
            .send_async_msg(UiAsyncMessage::UpdateSettings(properties));
    })
}
