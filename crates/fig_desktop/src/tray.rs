use fig_util::consts::PRODUCT_NAME;
use muda::{
    Menu,
    MenuEvent,
    MenuId,
    MenuItemBuilder,
    PredefinedMenuItem,
};
use tao::event_loop::ControlFlow;
use tracing::{
    error,
    trace,
};
use tray_icon::{
    Icon,
    TrayIcon,
    TrayIconBuilder,
};

use crate::event::{
    Event,
    WindowEvent,
};
use crate::{
    DASHBOARD_ID,
    EventLoopProxy,
};

const SETTINGS_ID: &str = "settings";
const TOGGLE_AUTOCOMPLETE_ID: &str = "toggle-autocomplete";
const QUIT_ID: &str = "quit";

const AUTOCOMPLETE_DISABLE_KEY: &str = "autocomplete.disable";

pub fn handle_event(menu_event: &MenuEvent, proxy: &EventLoopProxy) {
    match &*menu_event.id().0 {
        SETTINGS_ID => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::Batch(vec![
                        WindowEvent::NavigateRelative {
                            path: "/autocomplete".into(),
                        },
                        WindowEvent::Show,
                    ]),
                })
                .unwrap();
        },
        TOGGLE_AUTOCOMPLETE_ID => {
            let disabled = fig_settings::settings::get_bool_or(AUTOCOMPLETE_DISABLE_KEY, false);
            if let Err(err) = fig_settings::settings::set_value(AUTOCOMPLETE_DISABLE_KEY, !disabled) {
                error!(%err, "Failed to toggle {AUTOCOMPLETE_DISABLE_KEY}");
            }
            proxy.send_event(Event::ReloadAccessibility).ok();
            proxy.send_event(Event::ReloadTray).ok();
        },
        QUIT_ID => {
            proxy.send_event(Event::ControlFlow(ControlFlow::Exit)).unwrap();
        },
        id => {
            trace!(?id, "Unhandled tray event");
        },
    }
}

pub fn build_tray() -> tray_icon::Result<TrayIcon> {
    TrayIconBuilder::new()
        .with_icon(get_icon())
        .with_icon_as_template(true)
        .with_menu(Box::new(get_context_menu()))
        .build()
}

pub fn get_icon() -> Icon {
    let image = image::load_from_memory(include_bytes!("../icons/icon-monochrome.png"))
        .expect("Failed to open icon path")
        .into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).expect("Failed to open icon")
}

pub fn get_context_menu() -> Menu {
    let autocomplete_enabled = !fig_settings::settings::get_bool_or(AUTOCOMPLETE_DISABLE_KEY, false);

    let item = |text: String, id: Option<&str>| {
        let builder = MenuItemBuilder::new().text(text).enabled(id.is_some());
        match id {
            Some(id) => builder.id(MenuId::new(id)).build(),
            None => builder.build(),
        }
    };

    let menu = Menu::new();
    let result = menu.append_items(&[
        &item("Settings…".into(), Some(SETTINGS_ID)),
        &item(
            format!(
                "Autocomplete: {}",
                if autocomplete_enabled { "Enabled" } else { "Disabled" }
            ),
            Some(TOGGLE_AUTOCOMPLETE_ID),
        ),
        &PredefinedMenuItem::separator(),
        &item(format!("Version {}", env!("CARGO_PKG_VERSION")), None),
        &item(format!("Quit {PRODUCT_NAME}"), Some(QUIT_ID)),
    ]);
    if let Err(err) = result {
        error!(%err, "Failed to build tray menu");
    }
    menu
}
