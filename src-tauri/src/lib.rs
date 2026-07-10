mod commands;
mod crypto;
mod db;
mod models;
mod ping;

use db::Database;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_autostart::ManagerExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let hidden_by_tray = std::sync::Arc::new(AtomicBool::new(false));
    let hbt_window = hidden_by_tray.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let app_dir: PathBuf = app.path().app_data_dir().expect("failed to get app data dir");
            let database = Database::new(app_dir).expect("failed to initialize database");
            let settings = database.get_settings();
            app.manage(database);

            let lang = &settings.lang;
            let (show_text, quit_text) = if lang == "pl" {
                ("Pokaż", "Zakończ")
            } else {
                ("Show", "Quit")
            };

            let show = MenuItemBuilder::with_id("show", show_text).build(app)?;
            let quit = MenuItemBuilder::with_id("quit", quit_text).build(app)?;
            let menu = MenuBuilder::new(app).item(&show).item(&quit).build()?;

            let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("failed to load tray icon");

            let hbt_tray = hidden_by_tray.clone();
            let hbt_menu = hidden_by_tray.clone();
            TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            if hbt_tray.load(Ordering::Relaxed) {
                                hbt_tray.store(false, Ordering::Relaxed);
                                let _ = window.show();
                                let _ = window.set_focus();
                            } else {
                                hbt_tray.store(true, Ordering::Relaxed);
                                let _ = window.hide();
                            }
                        }
                    }
                })
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        hbt_menu.store(false, Ordering::Relaxed);
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // window.show() is called from JS after restoring saved state

            if settings.start_with_system {
                let _ = app.autolaunch().enable();
            }

            Ok(())
        })
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if let Some(database) = window.try_state::<Database>() {
                    let settings = database.get_settings();
                    if settings.close_to_tray {
                        api.prevent_close();
                        hbt_window.store(true, Ordering::Relaxed);
                        let _ = window.hide();
                        return;
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_connections,
            commands::get_connection,
            commands::create_connection,
            commands::update_connection,
            commands::delete_connection,
            commands::duplicate_connection,
            commands::set_favorite,
            commands::launch_connection,
            commands::list_groups,
            commands::create_group,
            commands::update_group,
            commands::delete_group,
            commands::update_group_order,
            commands::get_settings,
            commands::save_settings,
            commands::list_tags,
            commands::rename_tag,
            commands::delete_tag,
            commands::ping_hosts,
            commands::export_data,
            commands::import_data,
            commands::save_export_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
