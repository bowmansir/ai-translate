use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use device_query::{DeviceQuery, DeviceState};
use enigo::{Enigo, Key, KeyboardControllable};
use serde_json::json;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, Position, State, WebviewUrl,
    WebviewWindowBuilder,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::{app_state::AppState, translation};

const POPUP_OFFSET_Y: i32 = 22;
const DEBOUNCE_MS: u64 = 200;

pub fn register_global_shortcut(app: &tauri::App) -> anyhow::Result<()> {
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, pressed_shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }

                if is_active_shortcut(app, pressed_shortcut) {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(_err) = handle_translate_shortcut(app).await {
                            #[cfg(debug_assertions)]
                            eprintln!("translate shortcut failed: {_err:?}");
                        }
                    });
                    return;
                }

                if is_show_main_shortcut(app, pressed_shortcut) {
                    show_main_window(app);
                }
            })
            .build(),
    )?;

    register_active_shortcut(app.handle())?;
    Ok(())
}

pub fn register_active_shortcut(app: &AppHandle) -> anyhow::Result<()> {
    let state: State<'_, AppState> = app.state();
    let settings = state.settings();
    let shortcut = parse_user_shortcut(&settings.shortcut)?;
    let main_shortcut = parse_user_shortcut(&settings.main_shortcut)?;

    app.global_shortcut().unregister_all()?;
    app.global_shortcut().register(shortcut)?;
    if shortcut != main_shortcut {
        app.global_shortcut().register(main_shortcut)?;
    }

    Ok(())
}

pub fn normalize_shortcut(input: &str) -> anyhow::Result<String> {
    let (modifiers, key) = parse_shortcut_parts(input)?;
    let mut parts = Vec::new();

    if modifiers.contains(Modifiers::CONTROL) {
        parts.push("Ctrl".to_string());
    }
    if modifiers.contains(Modifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if modifiers.contains(Modifiers::SHIFT) {
        parts.push("Shift".to_string());
    }
    parts.push(key);

    Ok(parts.join("+"))
}

fn parse_user_shortcut(input: &str) -> anyhow::Result<Shortcut> {
    let (modifiers, key) = parse_shortcut_parts(input)?;
    Ok(Shortcut::new(Some(modifiers), key_to_code(&key)?))
}

fn parse_shortcut_parts(input: &str) -> anyhow::Result<(Modifiers, String)> {
    let mut modifiers = Modifiers::empty();
    let mut key: Option<String> = None;

    for raw_part in input.split('+') {
        let part = raw_part.trim();
        if part.is_empty() {
            continue;
        }

        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            candidate => {
                if key.is_some() {
                    anyhow::bail!("快捷键只能包含一个主键");
                }
                key = Some(candidate.to_ascii_uppercase());
            }
        }
    }

    if !modifiers.intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT) {
        anyhow::bail!("快捷键至少需要包含 Alt、Ctrl 或 Shift 之一");
    }

    let key = key.ok_or_else(|| anyhow::anyhow!("快捷键需要一个字母或数字"))?;
    key_to_code(&key)?;
    Ok((modifiers, key))
}

fn key_to_code(key: &str) -> anyhow::Result<Code> {
    match key {
        "A" => Ok(Code::KeyA),
        "B" => Ok(Code::KeyB),
        "C" => Ok(Code::KeyC),
        "D" => Ok(Code::KeyD),
        "E" => Ok(Code::KeyE),
        "F" => Ok(Code::KeyF),
        "G" => Ok(Code::KeyG),
        "H" => Ok(Code::KeyH),
        "I" => Ok(Code::KeyI),
        "J" => Ok(Code::KeyJ),
        "K" => Ok(Code::KeyK),
        "L" => Ok(Code::KeyL),
        "M" => Ok(Code::KeyM),
        "N" => Ok(Code::KeyN),
        "O" => Ok(Code::KeyO),
        "P" => Ok(Code::KeyP),
        "Q" => Ok(Code::KeyQ),
        "R" => Ok(Code::KeyR),
        "S" => Ok(Code::KeyS),
        "T" => Ok(Code::KeyT),
        "U" => Ok(Code::KeyU),
        "V" => Ok(Code::KeyV),
        "W" => Ok(Code::KeyW),
        "X" => Ok(Code::KeyX),
        "Y" => Ok(Code::KeyY),
        "Z" => Ok(Code::KeyZ),
        "0" => Ok(Code::Digit0),
        "1" => Ok(Code::Digit1),
        "2" => Ok(Code::Digit2),
        "3" => Ok(Code::Digit3),
        "4" => Ok(Code::Digit4),
        "5" => Ok(Code::Digit5),
        "6" => Ok(Code::Digit6),
        "7" => Ok(Code::Digit7),
        "8" => Ok(Code::Digit8),
        "9" => Ok(Code::Digit9),
        _ => anyhow::bail!("快捷键主键只支持 A-Z 或 0-9"),
    }
}

fn is_active_shortcut(app: &AppHandle, pressed_shortcut: &Shortcut) -> bool {
    let state: State<'_, AppState> = app.state();
    let settings = state.settings();
    parse_user_shortcut(&settings.shortcut).is_ok_and(|shortcut| pressed_shortcut == &shortcut)
}

fn is_show_main_shortcut(app: &AppHandle, pressed_shortcut: &Shortcut) -> bool {
    let state: State<'_, AppState> = app.state();
    let settings = state.settings();
    parse_user_shortcut(&settings.main_shortcut).is_ok_and(|shortcut| pressed_shortcut == &shortcut)
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn ensure_popup_window(app: &AppHandle) -> tauri::Result<tauri::WebviewWindow> {
    if let Some(window) = app.get_webview_window("popup") {
        return Ok(window);
    }

    WebviewWindowBuilder::new(app, "popup", WebviewUrl::App("popup.html".into()))
        .title("AI 翻译")
        .inner_size(520.0, 360.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focusable(true)
        .shadow(false)
        .build()
}

fn release_active_shortcut_keys(app: &AppHandle) {
    let state: State<'_, AppState> = app.state();
    let settings = state.settings();
    let Ok((modifiers, key)) = parse_shortcut_parts(&settings.shortcut) else {
        return;
    };

    let mut enigo = Enigo::new();
    if modifiers.contains(Modifiers::ALT) {
        enigo.key_up(Key::Alt);
    }
    if modifiers.contains(Modifiers::CONTROL) {
        enigo.key_up(Key::Control);
    }
    if modifiers.contains(Modifiers::SHIFT) {
        enigo.key_up(Key::Shift);
    }
    if let Some(ch) = key.chars().next() {
        enigo.key_up(Key::Layout(ch.to_ascii_lowercase()));
    }
}

pub async fn trigger_from_selection(
    app: AppHandle,
) -> anyhow::Result<translation::TranslationPayload> {
    let state: State<'_, AppState> = app.state();

    if !state.should_accept_trigger(Duration::from_millis(DEBOUNCE_MS)) {
        anyhow::bail!("debounced");
    }

    let clipboard_text = copy_selected_text(&app).await;
    let text = clipboard_text.trim();
    if text.is_empty() {
        show_popup_error(
            &app,
            "未读取到选中文本。Alt+D 在浏览器中可能会先聚焦地址栏；请确认文字仍被选中，或在设置中改用不冲突的快捷键。".to_string(),
        )?;
        anyhow::bail!("selected text is empty");
    }

    show_translation_near_mouse(app, text.to_string()).await
}

pub async fn trigger_with_sample(
    app: AppHandle,
) -> anyhow::Result<translation::TranslationPayload> {
    show_translation_near_mouse(app, sample_text().to_string()).await
}

async fn handle_translate_shortcut(app: AppHandle) -> anyhow::Result<()> {
    let _ = trigger_from_selection(app).await?;
    Ok(())
}

async fn read_clipboard_text_with_retry(app: &AppHandle, sentinel: &str) -> String {
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(45)).await;
        let clipboard_text = app.clipboard().read_text().unwrap_or_default();
        if !clipboard_text.trim().is_empty() && clipboard_text != sentinel {
            return clipboard_text;
        }
    }

    String::new()
}

async fn copy_selected_text(app: &AppHandle) -> String {
    let previous_clipboard = app.clipboard().read_text().unwrap_or_default();
    let sentinel = format!(
        "__AI_TRANSLATE_COPY_SENTINEL_{}__",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    );

    release_active_shortcut_keys(app);
    tokio::time::sleep(Duration::from_millis(120)).await;

    for attempt in 0..3 {
        let _ = app.clipboard().write_text(sentinel.clone());
        tokio::time::sleep(Duration::from_millis(35 + attempt * 40)).await;
        simulate_copy();

        let clipboard_text = read_clipboard_text_with_retry(app, &sentinel).await;
        if !clipboard_text.trim().is_empty() {
            return clipboard_text;
        }

        tokio::time::sleep(Duration::from_millis(120)).await;
    }

    let _ = app.clipboard().write_text(previous_clipboard);
    String::new()
}

fn show_popup_error(app: &AppHandle, message: String) -> anyhow::Result<()> {
    let mouse = current_mouse_position();
    let shortcut = app.state::<AppState>().settings().shortcut;
    let popup_payload = json!({
        "surface": "popup",
        "source": "",
        "target": message,
        "latency": 0,
        "provider": "selection",
        "shortcut": shortcut,
        "source_lang": "auto",
        "target_lang": "zh-CN",
    });
    app.state::<AppState>().set_popup_payload(popup_payload);
    let window = ensure_popup_window(app)?;
    window.set_position(Position::Physical(PhysicalPosition {
        x: mouse.0,
        y: mouse.1 + POPUP_OFFSET_Y,
    }))?;
    window.show()?;
    window.set_focus()?;
    thread::sleep(Duration::from_millis(60));
    window.emit(
        "translation-ready",
        json!({
            "surface": "popup",
            "source": "",
            "target": message,
            "latency": 0,
            "provider": "selection",
            "shortcut": app.state::<AppState>().settings().shortcut,
            "source_lang": "auto",
            "target_lang": "zh-CN",
        }),
    )?;

    Ok(())
}

async fn show_translation_near_mouse(
    app: AppHandle,
    source: String,
) -> anyhow::Result<translation::TranslationPayload> {
    let mouse = current_mouse_position();
    let state: State<'_, AppState> = app.state();
    let provider_config = state.provider_config();
    state.set_popup_payload(json!({
        "surface": "popup",
        "source": source.clone(),
        "target": "翻译中...",
        "latency": 0,
        "provider": provider_config.name,
        "shortcut": state.settings().shortcut,
        "source_lang": "auto",
        "target_lang": "zh-CN",
    }));

    let window = ensure_popup_window(&app)?;
    window.set_position(Position::Physical(PhysicalPosition {
        x: mouse.0,
        y: mouse.1 + POPUP_OFFSET_Y,
    }))?;
    window.show()?;
    window.set_focus()?;
    tokio::time::sleep(Duration::from_millis(60)).await;
    window.emit(
        "translation-ready",
        json!({
            "surface": "popup",
            "source": source.clone(),
            "target": "翻译中...",
            "latency": 0,
            "provider": provider_config.name,
            "source_lang": "auto",
            "target_lang": "zh-CN",
        }),
    )?;

    let payload = match translation::translate_text_with_provider(
        source,
        "auto".to_string(),
        "zh-CN".to_string(),
        &provider_config,
    )
    .await
    {
        Ok(payload) => payload,
        Err(err) => {
            let error_payload = json!({
                "surface": "popup",
                "source": "",
                "target": err.to_string(),
                "latency": 0,
                "provider": provider_config.name,
                "shortcut": state.settings().shortcut,
                "source_lang": "auto",
                "target_lang": "zh-CN",
            });
            app.state::<AppState>()
                .set_popup_payload(error_payload.clone());
            let window = ensure_popup_window(&app)?;
            window.emit("translation-ready", error_payload.clone())?;
            return Err(err);
        }
    };

    let window = ensure_popup_window(&app)?;
    let popup_payload = json!({
        "surface": "popup",
        "source": payload.source.clone(),
        "target": payload.target.clone(),
        "latency": payload.latency,
        "provider": payload.provider.clone(),
        "shortcut": state.settings().shortcut,
        "source_lang": payload.source_lang.clone(),
        "target_lang": payload.target_lang.clone(),
    });
    app.state::<AppState>()
        .set_popup_payload(popup_payload.clone());
    window.emit("translation-ready", popup_payload)?;

    Ok(payload)
}

fn simulate_copy() {
    thread::spawn(|| {
        let mut enigo = Enigo::new();

        #[cfg(target_os = "macos")]
        {
            enigo.key_down(Key::Meta);
            enigo.key_click(Key::Layout('c'));
            enigo.key_up(Key::Meta);
        }

        #[cfg(not(target_os = "macos"))]
        {
            enigo.key_down(Key::Control);
            thread::sleep(Duration::from_millis(20));
            enigo.key_down(Key::Layout('c'));
            thread::sleep(Duration::from_millis(20));
            enigo.key_up(Key::Layout('c'));
            thread::sleep(Duration::from_millis(20));
            enigo.key_up(Key::Control);
        }
    })
    .join()
    .ok();
}

fn current_mouse_position() -> (i32, i32) {
    let device_state = DeviceState::new();
    let mouse = device_state.get_mouse();
    (mouse.coords.0, mouse.coords.1)
}

fn sample_text() -> &'static str {
    "within a few milliseconds after the shortcut is pressed."
}
