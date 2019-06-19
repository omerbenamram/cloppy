use crate::file_listing::FilesMsg;
use crate::gui::event::Event;
use crate::gui::Wnd;
use crate::plugin::CustomDrawResult;
use crate::plugin::DrawResult;
use crate::plugin::Plugin;
use crate::plugin::State;
use crate::settings::Setting;
use crossbeam_channel::internal::channel;
use std::collections::HashMap;
use std::sync::Arc;
use winapi::um::winnt::LPWSTR;

pub struct GuiDispatcher {
    plugin: Arc<dyn Plugin>,
    state: Box<State>,
    sender: channel::Sender<UiAsyncMessage>,
}

impl GuiDispatcher {
    pub fn new(
        plugin: Arc<dyn Plugin>,
        state: Box<State>,
        sender: channel::Sender<UiAsyncMessage>,
    ) -> GuiDispatcher {
        GuiDispatcher {
            plugin,
            state,
            sender,
        }
    }

    pub fn prepare_item(&mut self, item_id: usize) {
        let state = &mut self.state;
        self.plugin.prepare_item(item_id, state);
    }

    pub fn custom_draw_item(&self, event: Event) -> CustomDrawResult {
        self.plugin.custom_draw_item(event, &self.state)
    }

    pub fn draw_item(&self, event: Event) -> DrawResult {
        self.plugin.draw_item(event, &self.state)
    }

    pub fn set_state(&mut self, state: Box<State>) {
        self.state = state;
    }

    //    pub fn send_msg(&self, msg: UiSyncMessage) -> UiResult {
    //        unimplemented!()
    //    }

    pub fn send_async_msg(&self, msg: UiAsyncMessage) {
        self.sender.send(msg);
    }
}

pub enum UiSyncMessage {
    DrawItem,
    CustomDrawItem,
    PrepareItem,
}

pub enum UiAsyncMessage {
    Start(Wnd),
    Ui(String),
    Files(FilesMsg),
    UpdateSettings(HashMap<Setting, String>),
}

pub enum UiResult {
    Ignore,
    Simple(LPWSTR),
    CustomHandled,
}
