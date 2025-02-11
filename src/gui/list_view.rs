use crate::dispatcher::GuiDispatcher;
use crate::errors::MyErrorKind::WindowsError;
use crate::gui::event::Event;
use crate::gui::get_string;
use crate::gui::list_header::ListHeader;
use crate::gui::wnd;
use crate::gui::Wnd;
use crate::gui::FILE_LIST_ID;
use crate::gui::FILE_LIST_NAME;
use crate::plugin::CustomDrawResult;
use crate::plugin::DrawResult;
use crate::plugin::State;
use failure::Error;
use failure::ResultExt;
use std::io;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::WC_LISTVIEW;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;

pub fn create(parent: Wnd, instance: Option<HINSTANCE>) -> ItemList {
    let (list, header) = new(parent, instance).unwrap();
    ItemList::new(list, header)
}

fn new(parent: Wnd, instance: Option<HINSTANCE>) -> Result<(wnd::Wnd, ListHeader), Error> {
    let list_view_params = wnd::WndParams::builder()
        .instance(instance)
        .window_name(get_string(FILE_LIST_NAME))
        .class_name(get_string(WC_LISTVIEW))
        .h_menu(FILE_LIST_ID as HMENU)
        .style(
            WS_VISIBLE
                | LVS_REPORT
                | LVS_SINGLESEL
                | LVS_OWNERDATA
                | LVS_ALIGNLEFT
                | LVS_SHAREIMAGELISTS
                | WS_CHILD,
        )
        .h_parent(parent.hwnd)
        .build();
    let list_view = wnd::Wnd::new(list_view_params)?;
    unsafe {
        SendMessageW(
            list_view.hwnd,
            LVM_SETEXTENDEDLISTVIEWSTYLE,
            (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as WPARAM,
            (LVS_EX_DOUBLEBUFFER | LVS_EX_FULLROWSELECT) as LPARAM,
        );
    };
    let header = ListHeader::create(&list_view);
    Ok((list_view, header))
}

pub struct ItemList {
    wnd: Wnd,
    header: ListHeader,
}

impl ItemList {
    fn new(wnd: Wnd, header: ListHeader) -> ItemList {
        ItemList { wnd, header }
    }

    pub fn header(&self) -> &ListHeader {
        &self.header
    }

    pub fn scroll_to_top(&self) {
        self.wnd.send_message(LVM_ENSUREVISIBLE, 0, false as isize);
    }

    pub fn wnd(&self) -> &Wnd {
        &self.wnd
    }

    pub fn on_header_click(&mut self, event: Event) {
        self.header.add_sort_arrow_to_header(event);
    }

    pub fn update(&self, state: &State) -> Result<(), Error> {
        self.scroll_to_top();
        match self
            .wnd
            .send_message(LVM_SETITEMCOUNT, state.count() as WPARAM, 0)
        {
            0 => {
                Err(io::Error::last_os_error()).context(WindowsError("LVM_SETITEMCOUNT failed"))?
            }
            _ => Ok(()),
        }
    }

    pub fn custom_draw(&mut self, event: Event, dispatcher: &mut GuiDispatcher) -> LRESULT {
        let custom_draw = event.as_custom_draw();
        const SUBITEM_PAINT: u32 = CDDS_SUBITEM | CDDS_ITEMPREPAINT;
        match custom_draw.nmcd.dwDrawStage {
            CDDS_PREPAINT => CDRF_NOTIFYITEMDRAW,
            CDDS_ITEMPREPAINT => {
                dispatcher.prepare_item(custom_draw.nmcd.dwItemSpec);
                CDRF_NOTIFYSUBITEMDRAW
            }
            SUBITEM_PAINT => match dispatcher.custom_draw_item(event) {
                CustomDrawResult::HANDLED => CDRF_SKIPDEFAULT,
                CustomDrawResult::IGNORED => CDRF_DODEFAULT,
            },
            _ => CDRF_DODEFAULT,
        }
    }

    pub fn display_item(&mut self, event: Event, dispatcher: &GuiDispatcher) {
        let item = &mut event.as_display_info().item;
        if (item.mask & LVIF_TEXT) == LVIF_TEXT {
            match dispatcher.draw_item(event) {
                DrawResult::SIMPLE(txt) => {
                    item.pszText = txt;
                }
                DrawResult::IGNORE => {}
            }
        }
    }
}
