use crate::actions::exit_app::exit_app;
use crate::actions::focus_on_input_field::focus_on_input_field;
use crate::actions::minimize_to_tray::minimize_to_tray;
use crate::actions::new_input_query::new_input_query;
use crate::actions::new_plugin_state::new_plugin_state;
use crate::actions::new_settings::new_settings;
use crate::actions::restore_columns_position::restore_columns_position;
use crate::actions::restore_windows_position::restore_windows_position;
use crate::actions::save_columns_position::save_columns_position;
use crate::actions::save_windows_position::save_windows_position;
use crate::actions::shortcuts::Shortcut;
use crate::actions::show_files_window::show_files_window;
use crate::errors::failure_to_string;
use crate::gui::event::Event;
use crate::gui::Gui;
use failure::Error;

mod exit_app;
mod focus_on_input_field;
mod minimize_to_tray;
mod new_input_query;
mod new_plugin_state;
mod new_settings;
mod restore_columns_position;
mod restore_windows_position;
mod save_columns_position;
mod save_windows_position;
pub mod shortcuts;
mod show_files_window;

#[derive(Copy, Clone, Debug)]
pub enum Action {
    Simple(SimpleAction),
    Composed(ComposedAction),
}

impl Action {
    pub fn execute(&self, event: Event, gui: &mut Gui) {
        debug!(&gui.logger(), "ui action" ; "action" => ?self);
        match self {
            Action::Simple(action) => {
                if let Err(e) = action.handler()(event, gui) {
                    error!(&gui.logger(), "ui action failed"; "action" => ?action, "error" => failure_to_string(e));
                }
            }
            Action::Composed(action) => {
                for simple_action in action.simple_actions() {
                    if let Err(e) = simple_action.handler()(event, gui) {
                        error!(&gui.logger(), "ui composed action failed"; "composed action" => ?action, "action" => ?simple_action, "error" => failure_to_string(e));
                        break;
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SimpleAction {
    ShowFilesWindow,
    NewInputQuery,
    ExitApp,
    MinimizeToTray,
    DoNothing,
    FocusOnInputField,
    SaveWindowPosition,
    RestoreWindowPosition,
    SaveColumnsPosition,
    RestoreColumnsPosition,
    NewPluginState,
    NewSettings,
    //    FocusOnItemList,
}

impl SimpleAction {
    pub fn handler(&self) -> impl Fn(Event, &mut Gui) -> Result<(), Error> {
        match self {
            SimpleAction::ShowFilesWindow => show_files_window,
            SimpleAction::MinimizeToTray => minimize_to_tray,
            SimpleAction::ExitApp => exit_app,
            SimpleAction::NewInputQuery => new_input_query,
            SimpleAction::FocusOnInputField => focus_on_input_field,
            SimpleAction::SaveWindowPosition => save_windows_position,
            SimpleAction::RestoreWindowPosition => restore_windows_position,
            SimpleAction::SaveColumnsPosition => save_columns_position,
            SimpleAction::RestoreColumnsPosition => restore_columns_position,
            SimpleAction::NewPluginState => new_plugin_state,
            SimpleAction::NewSettings => new_settings,
            SimpleAction::DoNothing => do_nothing,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ComposedAction {
    RestoreWindow,
    ResizeWindowFromSettings,
}

impl ComposedAction {
    pub fn simple_actions(self) -> &'static [SimpleAction] {
        static RESTORE_WINDOW: [SimpleAction; 2] = [
            SimpleAction::ShowFilesWindow,
            SimpleAction::FocusOnInputField,
        ];
        static RESIZE_WINDOW_FROM_SETTINGS: [SimpleAction; 2] = [
            SimpleAction::RestoreWindowPosition,
            SimpleAction::RestoreColumnsPosition,
        ];
        match self {
            ComposedAction::RestoreWindow => &RESTORE_WINDOW,
            ComposedAction::ResizeWindowFromSettings => &RESIZE_WINDOW_FROM_SETTINGS,
        }
    }
}

impl From<Shortcut> for Action {
    fn from(shortcut: Shortcut) -> Self {
        match shortcut {
            Shortcut::RestoreWindow => Action::Composed(ComposedAction::RestoreWindow),
        }
    }
}

impl From<SimpleAction> for Action {
    fn from(action: SimpleAction) -> Self {
        Action::Simple(action)
    }
}

impl From<ComposedAction> for Action {
    fn from(action: ComposedAction) -> Self {
        Action::Composed(action)
    }
}

fn do_nothing(_event: Event, _gui: &mut Gui) -> Result<(), Error> {
    Ok(())
}
