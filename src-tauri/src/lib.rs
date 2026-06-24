mod app_state;
mod shortcut;
mod translation;

use app_state::{AppState, ProviderConfig, ProviderStateView, RuntimeSettings};
use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Serialize)]
struct AppStatus {
    autostart_enabled: bool,
    shortcut: String,
    main_shortcut: String,
    startup_mode: String,
    theme: String,
    provider: String,
    ai_configured: bool,
}

#[tauri::command]
async fn simulate_translation(
    app: tauri::AppHandle,
) -> Result<translation::TranslationPayload, String> {
    shortcut::trigger_with_sample(app)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn translate_manual(
    app_state: tauri::State<'_, AppState>,
    text: String,
    source_lang: String,
    target_lang: String,
) -> Result<translation::TranslationPayload, String> {
    let provider_config = app_state.provider_config();
    translation::translate_text_with_provider(text, source_lang, target_lang, &provider_config)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn copy_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("没有可复制的译文".to_string());
    }

    app.clipboard()
        .write_text(text.to_string())
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn window_action(app: tauri::AppHandle, action: String) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    match action.as_str() {
        "minimize" => window.minimize(),
        "toggleMaximize" => {
            if window.is_maximized().map_err(|err| err.to_string())? {
                window.unmaximize()
            } else {
                window.maximize()
            }
        }
        "close" => window.hide(),
        _ => return Err("unsupported window action".to_string()),
    }
    .map_err(|err| err.to_string())
}

#[tauri::command]
fn get_app_status(app: tauri::AppHandle) -> Result<AppStatus, String> {
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);

    let state = app.state::<AppState>();
    let provider_config = state.provider_config();
    let settings = state.settings();

    let status = AppStatus {
        autostart_enabled,
        shortcut: settings.shortcut,
        main_shortcut: settings.main_shortcut,
        startup_mode: settings.startup_mode,
        theme: settings.theme,
        provider: provider_config.name,
        ai_configured: provider_config
            .api_key
            .as_ref()
            .is_some_and(|key| !key.trim().is_empty()),
    };
    Ok(status)
}

#[tauri::command]
fn get_runtime_settings(app_state: tauri::State<'_, AppState>) -> RuntimeSettings {
    app_state.settings()
}

#[tauri::command]
fn get_popup_payload(app_state: tauri::State<'_, AppState>) -> Option<serde_json::Value> {
    app_state.popup_payload()
}

#[tauri::command]
fn set_runtime_settings(
    app: tauri::AppHandle,
    app_state: tauri::State<'_, AppState>,
    mut settings: RuntimeSettings,
) -> Result<RuntimeSettings, String> {
    settings.shortcut =
        shortcut::normalize_shortcut(&settings.shortcut).map_err(|err| err.to_string())?;
    settings.main_shortcut =
        shortcut::normalize_shortcut(&settings.main_shortcut).map_err(|err| err.to_string())?;
    let saved = app_state
        .update_settings(&app, settings)
        .map_err(|err| err.to_string())?;
    shortcut::register_active_shortcut(&app).map_err(|err| err.to_string())?;
    Ok(saved)
}

#[tauri::command]
fn get_provider_state(app_state: tauri::State<'_, AppState>) -> ProviderStateView {
    app_state.provider_state_view()
}

#[tauri::command]
fn save_provider_config(
    app: tauri::AppHandle,
    app_state: tauri::State<'_, AppState>,
    config: ProviderConfig,
) -> Result<ProviderStateView, String> {
    app_state
        .save_provider_config(&app, config)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn set_active_provider(
    app: tauri::AppHandle,
    app_state: tauri::State<'_, AppState>,
    provider_id: String,
) -> Result<ProviderStateView, String> {
    app_state
        .set_active_provider(&app, provider_id)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<AppStatus, String> {
    let manager = app.autolaunch();

    if enabled {
        manager.enable().map_err(|err| err.to_string())?;
    } else {
        if let Err(error) = manager.disable() {
            let message = error.to_string();
            if !message.contains("os error 2") && !message.contains("找不到指定的文件") {
                return Err(message);
            }
        }
    }

    get_app_status(app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                let _ = show_main_window(app);
            }))
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            ));
    }

    builder
        .manage(AppState::new())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            simulate_translation,
            translate_manual,
            get_app_status,
            copy_text,
            window_action,
            set_autostart,
            get_provider_state,
            save_provider_config,
            set_active_provider,
            get_runtime_settings,
            get_popup_payload,
            set_runtime_settings
        ])
        .setup(|app| {
            if let Err(error) = app.state::<AppState>().load_from_disk(app.handle()) {
                eprintln!("failed to load app config: {error}");
            }

            #[cfg(desktop)]
            {
                shortcut::register_global_shortcut(app)?;
                create_tray(app)?;
                let settings = app.state::<AppState>().settings();
                ensure_main_window(app.handle(), settings.startup_mode == "main")?;
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "popup" {
                if let WindowEvent::Focused(false) = event {
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn ensure_main_window(app: &tauri::AppHandle, show: bool) -> tauri::Result<()> {
    let window = if let Some(window) = app.get_webview_window("main") {
        window
    } else {
        WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
            .title("AI 翻译")
            .inner_size(920.0, 600.0)
            .resizable(true)
            .decorations(false)
            .focusable(true)
            .visible(show)
            .build()?
    };

    if show {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    } else {
        window.hide()?;
    }

    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    ensure_main_window(app, true)
}

fn create_tray(app: &tauri::App) -> tauri::Result<()> {
    let show_main = MenuItem::with_id(app, "show_main", "显示主界面", true, None::<&str>)?;
    let hide_popup = MenuItem::with_id(app, "hide_popup", "隐藏翻译窗", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_main, &hide_popup, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("AI 翻译")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_main" => {
                let _ = show_main_window(app);
            }
            "hide_popup" => {
                if let Some(window) = app.get_webview_window("popup") {
                    let _ = window.hide();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = show_main_window(app);
            }
        })
        .build(app)?;

    Ok(())
}
