#![allow(dead_code)]
#![recursion_limit = "1024"]

#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate conv;
extern crate core;
extern crate crossbeam_channel;
#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate failure;
extern crate ini;
#[macro_use]
extern crate lazy_static;
extern crate num;
extern crate parking_lot;
extern crate rayon;
#[macro_use]
extern crate rusqlite;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate time;
extern crate twoway;
#[macro_use]
extern crate typed_builder;
extern crate winapi;

use crate::dispatcher::GuiDispatcher;
use crate::dispatcher::UiAsyncMessage;
use crate::errors::failure_to_string;
use crate::errors::MyErrorKind::UserSettingsError;
use crate::gui::GuiCreateParams;
use crate::gui::Wnd;
use crate::plugin::Plugin;
use crate::plugin::State;
use crate::plugin_handler::PluginHandler;
use crate::settings::UserSettings;
use crossbeam_channel as channel;
use failure::Error;
use failure::ResultExt;
use std::sync::Arc;
use std::thread;

mod actions;
mod dispatcher;
mod errors;
pub mod file_listing;
mod gui;
mod logger;
mod ntfs;
mod plugin;
mod plugin_handler;
mod resources;
mod settings;
mod sql;
mod windows;

fn main() {
    let logger = logger::setup();
    let result = ntfs::parse_operation::run(logger.clone())
        .and_then(|_| try_main(logger.clone()))
        .map_err(failure_to_string);
    match result {
        Ok(code) => ::std::process::exit(code),
        Err(msg) => error!(logger, "Error: {}", msg),
    }
}

fn try_main(logger: slog::Logger) -> Result<i32, Error> {
    let settings = UserSettings::load(logger.clone()).context(UserSettingsError)?;
    let (req_snd, req_rcv) = channel::unbounded();
    let arena = sql::load_all_arena().unwrap();
    let files = Arc::new(file_listing::FileListing::create(
        arena,
        req_snd.clone(),
        &logger,
    ));
    let state = State::new("", 0, files.default_plugin_state());

    let logger_ui = logger.new(o!("thread" => "ui"));
    let dispatcher_ui = GuiDispatcher::new(files.clone(), Box::new(state.clone()), req_snd);
    let settings_ui = settings.get_settings();
    thread::Builder::new()
        .name("producer".to_string())
        .spawn(move || {
            let gui_params = GuiCreateParams {
                logger: Arc::into_raw(Arc::new(logger_ui)),
                dispatcher: Box::into_raw(Box::new(dispatcher_ui)),
                settings: Box::into_raw(Box::new(settings_ui)),
            };
            gui::init_wingui(gui_params).unwrap()
        })
        .unwrap();
    let wnd = wait_for_wnd(req_rcv.clone()).expect("Didnt receive START msg with main_wnd");
    let mut handler = PluginHandler::new(wnd, files, state);
    handler.run_forever(req_rcv, settings);
    Ok(0)
}

fn wait_for_wnd(receiver: channel::Receiver<UiAsyncMessage>) -> Option<Wnd> {
    loop {
        let msg = match receiver.recv() {
            Some(e) => e,
            None => {
                println!("Channel closed. Probably UI thread exit.");
                return None;
            }
        };
        if let UiAsyncMessage::Start(wnd) = msg {
            println!("Got wnd");
            return Some(wnd);
        }
    }
}
