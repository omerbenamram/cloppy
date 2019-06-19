use crate::dispatcher::UiAsyncMessage;
use crate::errors::MyErrorKind::WindowsError;
use crate::gui::event::Event;
use crate::gui::Gui;
use crate::gui::Wnd;
use crate::settings::Setting;
use failure::Error;
use failure::ResultExt;
use std::collections::HashMap;
use std::io;
use std::string::ToString;
use winapi::shared::minwindef::WPARAM;
use winapi::um::commctrl::HDM_GETITEMCOUNT;
use winapi::um::commctrl::LVM_GETCOLUMNWIDTH;

pub fn save_columns_position(_event: Event, gui: &mut Gui) -> Result<(), Error> {
    let item_count = get_column_count(gui.item_list().header().wnd())?;
    assert_eq!(item_count, 3);
    let mut properties = HashMap::new();
    for index in 0..item_count {
        let (setting, value) = get_item(index, gui.item_list().wnd())?;
        properties.insert(setting, value);
    }
    gui.dispatcher()
        .send_async_msg(UiAsyncMessage::UpdateSettings(properties));
    Ok(())
}

fn get_column_count(wnd: &Wnd) -> Result<isize, Error> {
    match wnd.send_message(HDM_GETITEMCOUNT, 0, 0) {
        -1 => Err(io::Error::last_os_error()).context(WindowsError("HDM_GETITEMCOUNT failed"))?,
        v => Ok(v),
    }
}

fn get_item(index: isize, wnd: &Wnd) -> Result<(Setting, String), Error> {
    let width: Result<isize, Error> = match wnd.send_message(LVM_GETCOLUMNWIDTH, index as WPARAM, 0)
    {
        v if v < 1 => Err(io::Error::last_os_error())
            .with_context(|e| format!("LVM_GETCOLUMNWIDTH failed - index {}: {}", index, e))?,
        v => Ok(v),
    };
    match index {
        0 => Ok((Setting::ColumnFileNameWidth, width?.to_string())),
        1 => Ok((Setting::ColumnFilePathWidth, width?.to_string())),
        2 => Ok((Setting::ColumnFileSizeWidth, width?.to_string())),
        _ => bail!("Wrong index - nonexistent column {}", index),
    }
}
