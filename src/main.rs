#![feature(plugin, custom_attribute, test)]
#![allow(dead_code)]
#![recursion_limit = "1024"]
#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate conv;
extern crate core;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate ini;
#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
extern crate regex;
extern crate rusqlite;
extern crate test;
extern crate time;
extern crate twoway;
#[macro_use]
extern crate typed_builder;
extern crate winapi;

use errors::failure_to_string;
use file_listing::files::Files;
use rusqlite::Connection;
use std::ffi::OsString;
use std::io;
use std::ops::Range;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

mod windows;
mod ntfs;
mod sql;
//mod user_settings;
mod errors;
mod gui;
mod resources;
pub mod file_listing;

fn main() {
//    let mut con = sql::main();
//    main1(&mut con);
//    sql::create_indices(&con);
    match try_main() {
        Ok(code) => ::std::process::exit(code),
        Err(err) => {
            let msg = format!("Error: {}", err);
            panic!(msg);
        }
    }
}

fn try_main() -> io::Result<i32> {
    let (req_snd, req_rcv) = mpsc::channel();
    let arena = sql::load_all_arena().unwrap();
//    arena.path_of(1274);
//    arena.set_paths();
    let now = Instant::now();
//    arena.sort_by_name();
    println!("total time {:?}", Instant::now().duration_since(now));
    let arena = Arc::new(arena);
    let arena_gui = arena.clone();
    thread::spawn(move || {
        gui::init_wingui(req_snd, arena_gui).unwrap();
    });
    run_forever(req_rcv, arena);
    Ok(0)
}

fn run_forever(receiver: mpsc::Receiver<Message>, arena: Arc<Files>) {
//    let con = sql::main();
//    let (tree, _) = sql::insert_tree().unwrap();
    let mut operation = file_listing::FileListing::new(arena);
    loop {
        let event = match receiver.recv() {
            Ok(e) => e,
            Err(_) => {
                println!("Channel closed. Probably UI thread exit.");
                return;
            }
        };
        operation.handle(event);
    }
}

fn main1(con: &mut Connection) {
    if let Err(e) = ntfs::start(con) {
        println!("{}", failure_to_string(e));
    }
}

pub enum Message {
    START(gui::Wnd),
    MSG(OsString),
    LOAD(Range<u32>),
}

pub enum StateChange {
    NEW,
    UPDATE,
}

impl Default for StateChange {
    fn default() -> Self {
        StateChange::NEW
    }
}
