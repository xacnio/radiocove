use std::path::PathBuf;
use tauri::{App, Manager};

use crate::platform;
use crate::services::media::MediaSession;
use crate::state::AppState;

/// Old app identifier, used to find/migrate data from the "Radiko Desktop" -> "Radiocove" rename.
const LEGACY_IDENTIFIER: &str = "dev.xacnio.radikodesktop";
const LEGACY_DISPLAY_NAME: &str = "Radiko Desktop";
/// Must match `identifier` in tauri.conf.json.
const CURRENT_IDENTIFIER: &str = "dev.xacnio.radiocove";

/// Checks for the '.pending_reset' flag and cleans up WebView/cache directories if found.
pub fn check_pending_reset() {
    if let Some(data_dir) = identifier_data_dir(CURRENT_IDENTIFIER) {
        let flag = data_dir.join(".pending_reset");
        if flag.exists() {
            tracing::info!("Pending reset detected, cleaning WebView data...");
            let _ = std::fs::remove_file(&flag);

            // Delete EBWebView (Windows WebView2 data)
            let webview_dir = data_dir.join("EBWebView");
            if webview_dir.exists() {
                tracing::info!("Deleting WebView data: {:?}", webview_dir);
                let _ = std::fs::remove_dir_all(&webview_dir);
            }

            // Delete cache directory
            if let Some(cache) = identifier_cache_dir(CURRENT_IDENTIFIER) {
                if cache.exists() {
                    tracing::info!("Deleting cache: {:?}", cache);
                    let _ = std::fs::remove_dir_all(&cache);
                }
            }
        }
    }
}

fn identifier_data_dir(identifier: &str) -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        std::env::var("APPDATA").ok().map(|d| PathBuf::from(d).join(identifier))
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .ok()
            .map(|d| PathBuf::from(d).join("Library/Application Support").join(identifier))
    } else {
        std::env::var("HOME")
            .ok()
            .map(|d| PathBuf::from(d).join(".local/share").join(identifier))
    }
}

fn identifier_cache_dir(identifier: &str) -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        std::env::var("LOCALAPPDATA").ok().map(|d| PathBuf::from(d).join(identifier))
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .ok()
            .map(|d| PathBuf::from(d).join("Library/Caches").join(identifier))
    } else {
        std::env::var("HOME")
            .ok()
            .map(|d| PathBuf::from(d).join(".cache").join(identifier))
    }
}

/// One-time migration from the previous "Radiko Desktop" install (identifier change on rename).
/// Copies user data (settings, custom stations, identified songs, uploaded favicons) from the
/// old data/cache directories into the new ones, then deletes the old directories. Must run
/// before `Settings::load()` so the migrated settings are picked up on this very launch.
pub fn migrate_legacy_data() {
    if let Some(old_data) = identifier_data_dir(LEGACY_IDENTIFIER) {
        if old_data.exists() {
            if let Some(new_data) = identifier_data_dir(CURRENT_IDENTIFIER) {
                if !new_data.join("settings.json").exists() {
                    tracing::info!("Migrating legacy app data: {:?} -> {:?}", old_data, new_data);
                    let _ = std::fs::create_dir_all(&new_data);
                    for fname in ["settings.json", "custom_stations.json", "identified_songs.json"] {
                        let src = old_data.join(fname);
                        if src.exists() {
                            if let Err(e) = std::fs::copy(&src, new_data.join(fname)) {
                                tracing::warn!("Failed to migrate {}: {:?}", fname, e);
                            }
                        }
                    }
                }
            }
            let _ = std::fs::remove_dir_all(&old_data);
        }
    }

    if let Some(old_cache) = identifier_cache_dir(LEGACY_IDENTIFIER) {
        if old_cache.exists() {
            if let Some(new_cache) = identifier_cache_dir(CURRENT_IDENTIFIER) {
                let _ = std::fs::create_dir_all(&new_cache);
                if let Ok(entries) = std::fs::read_dir(&old_cache) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if !path.is_file() {
                            continue;
                        }
                        // Copy every cached image, not just "custom_*": downloaded covers
                        // ("cover_<hash>.png") are also referenced by file:// paths persisted in
                        // custom_stations.json, so skipping them breaks every station's favicon.
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            let dest = new_cache.join(name);
                            if !dest.exists() {
                                let _ = std::fs::copy(&path, &dest);
                            }
                        }
                    }
                }
            }
            let _ = std::fs::remove_dir_all(&old_cache);
        }
    }
}

/// Windows only: silently runs the previous "Radiko Desktop" installer's uninstaller (if still
/// registered) and removes its stale Start Menu/Desktop shortcuts. Needed because the NSIS
/// installer identifier changed with the rename, so Windows treats Radiocove as a separate
/// product and won't clean up the old install on its own.
#[cfg(target_os = "windows")]
pub fn cleanup_legacy_windows_install() {
    use std::os::windows::process::CommandExt;
    use tracing::{info, warn};

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let ps_script = format!(
        r#"
$keys = @(
    'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
    'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
    'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*'
)
foreach ($k in $keys) {{
    Get-ItemProperty $k -ErrorAction SilentlyContinue | Where-Object {{ $_.DisplayName -eq '{}' }} | ForEach-Object {{
        if ($_.QuietUninstallString) {{ Write-Output $_.QuietUninstallString }}
        elseif ($_.UninstallString) {{ Write-Output ($_.UninstallString + ' /S') }}
    }}
}}
"#,
        LEGACY_DISPLAY_NAME.replace('\'', "''")
    );

    match std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().map(str::trim).filter(|l| !l.is_empty()) {
                info!("Found legacy '{}' install, uninstalling: {}", LEGACY_DISPLAY_NAME, line);
                match std::process::Command::new("cmd")
                    .args(["/C", line])
                    .creation_flags(CREATE_NO_WINDOW)
                    .status()
                {
                    Ok(status) => info!("Legacy uninstaller exited with: {:?}", status.code()),
                    Err(e) => warn!("Failed to run legacy uninstaller: {:?}", e),
                }
            }
        }
        Err(e) => warn!("Failed to query legacy uninstall registry: {:?}", e),
    }

    // Remove stale shortcuts in case the old uninstaller didn't (or wasn't found at all).
    let shortcut_name = format!("{}.lnk", LEGACY_DISPLAY_NAME);
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata)
            .join("Microsoft/Windows/Start Menu/Programs")
            .join(&shortcut_name);
        if p.exists() {
            info!("Removing stale Start Menu shortcut: {:?}", p);
            let _ = std::fs::remove_file(&p);
        }
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(userprofile).join("Desktop").join(&shortcut_name);
        if p.exists() {
            info!("Removing stale Desktop shortcut: {:?}", p);
            let _ = std::fs::remove_file(&p);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn cleanup_legacy_windows_install() {}

/// Creates a default cover image ("default_cover_v2.png") for stations without a favicon.
pub fn generate_default_cover(app: &App) {
    if let Ok(cache_dir) = app.path().app_cache_dir() {
        let _ = std::fs::create_dir_all(&cache_dir);
        let default_cover_path = cache_dir.join("default_cover_v2.png");

        if !default_cover_path.exists() {
            // Generate a beautiful vinyl record aesthetic cover
            let img = image::RgbaImage::from_fn(256, 256, |x, y| {
                let cx = (x as f32 - 128.0) / 128.0;
                let cy = (y as f32 - 128.0) / 128.0;
                let d = (cx * cx + cy * cy).sqrt();

                if d > 0.95 {
                    // Background corner gradient
                    let bg = (15.0 * (1.5 - d).max(0.0)) as u8;
                    image::Rgba([bg, bg, bg, 255])
                } else if d > 0.35 {
                    // Vinyl grooves with slight light reflection
                    let angle = cy.atan2(cx);
                    let reflection = (angle * 2.0).sin().powi(4) * 15.0; // Shiny diagonal light
                    let groove = (d * 180.0).sin() * 6.0;
                    let base = (22.0 + groove + reflection).clamp(0.0, 255.0) as u8;
                    image::Rgba([base, base, base, 255])
                } else if d > 0.06 {
                    // Center label (Dynamic Accent Green)
                    let green_shade = (185.0 - (d * 80.0)).clamp(0.0, 255.0) as u8;
                    image::Rgba([29, green_shade, 84, 255])
                } else {
                    // Center hole
                    image::Rgba([10, 10, 10, 255])
                }
            });
            let _ = img.save(&default_cover_path);
        }

        let path_str = default_cover_path.to_string_lossy().replace('\\', "/");
        let cover_str = if path_str.starts_with('/') {
            format!("file://{}", path_str)
        } else {
            format!("file:///{}", path_str)
        };
        if let Some(app_state) = app.try_state::<AppState>() {
            if let Ok(mut ps) = app_state.inner.lock() {
                ps.default_cover = Some(cover_str);
            }
        }
    }
}

/// On macOS/Linux, creates a static HTML splash window (Tauri-managed).
#[cfg(not(target_os = "windows"))]
pub fn setup_html_splash(app: &mut App, theme: Option<&str>) {
    let is_light = match theme {
        Some("light") => true,
        Some("dark") => false,
        _ => dark_light::detect() == dark_light::Mode::Light,
    };

    let bg_color = if is_light {
        (245, 245, 245, 255)
    } else {
        (13, 13, 13, 255)
    };

    let _ = tauri::WebviewWindowBuilder::new(
        app,
        "splash",
        tauri::WebviewUrl::App("splash.html".into()),
    )
    .title("Radiocove")
    .inner_size(340.0, 148.0)
    .center()
    .decorations(false)
    .background_color(bg_color.into())
    .always_on_top(true)
    .resizable(false)
    .build();
}

/// Same as `setup_html_splash` but works with an `AppHandle` (used when recreating the main window).
#[cfg(not(target_os = "windows"))]
fn show_html_splash_handle(app: &tauri::AppHandle) {
    let is_light = dark_light::detect() == dark_light::Mode::Light;
    let bg_color: (u8, u8, u8, u8) = if is_light { (245, 245, 245, 255) } else { (13, 13, 13, 255) };
    let _ = tauri::WebviewWindowBuilder::new(
        app,
        "splash",
        tauri::WebviewUrl::App("splash.html".into()),
    )
    .title("Radiocove")
    .inner_size(340.0, 148.0)
    .center()
    .decorations(false)
    .background_color(bg_color.into())
    .always_on_top(true)
    .resizable(false)
    .build();
}

/// Initialises OS media transport controls (Windows SMTC or macOS Now Playing) and event listeners
/// for the "main" window. Called once at startup, and again every time "main" is recreated after
/// being destroyed by the idle-destroy poller (see `get_or_create_main_window`) — Windows SMTC and
/// the thumbnail toolbar are bound to the window's HWND, which changes on every recreate.
pub fn attach_main_window_listeners(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        // Set AppUserModelID so Windows SMTC shows "Radiocove" instead of "Unknown app"
        extern "system" {
            fn SetCurrentProcessExplicitAppUserModelID(app_id: *const u16) -> i32;
        }
        let app_id: Vec<u16> = "dev.xacnio.radiocove\0".encode_utf16().collect();
        unsafe {
            SetCurrentProcessExplicitAppUserModelID(app_id.as_ptr());
        }

        // Create Start Menu shortcut with AppUserModelID
        // so that SMTC can resolve the app name from the shortcut
        if !cfg!(debug_assertions) {
            platform::shortcut::ensure_start_menu_shortcut(
                "dev.xacnio.radiocove",
                "Radiocove",
            );
        }

        if let Ok(hwnd) = window.hwnd() {
            // Add thumbnail toolbar buttons (prev/play-pause/next) — plain Win32, safe on this thread.
            platform::thumbbar::setup_thumb_buttons(hwnd.0 as isize, app.clone());

            // SMTC re-init (MediaSession::new) goes through WinRT and appears to hang when
            // re-run on the same thread that's mid-event-loop (e.g. a tray click handler) —
            // only startup, where this runs before the event loop is pumping, is reliably
            // safe inline. Run it on its own thread so a hang there can no longer freeze the
            // window-recreate flow (splash close, tray clicks) on every idle-destroy recreate.
            let hwnd_isize = hwnd.0 as isize;
            let app_clone = app.clone();
            std::thread::spawn(move || {
                let hwnd_ptr = hwnd_isize as *mut std::ffi::c_void;
                let session = MediaSession::new(hwnd_ptr, app_clone.clone());
                if session.is_some() {
                    *app_clone.state::<AppState>().media_session.lock().unwrap() = session;
                }
            });
        }

        // Off-screen / wrong-size fix: Win+D with decorations:false causes the window to
        // restore with a corrupted size/position. Save last known good geometry (persisted
        // in AppState so it survives a destroy/recreate cycle) and restore it if corrupted.
        let win = window.clone();
        let app_handle = app.clone();
        window.on_window_event(move |event| match event {
            tauri::WindowEvent::Resized(_) => {
                if win.is_minimized().unwrap_or(false) {
                    return;
                }
                if let (Ok(size), Ok(pos)) = (win.outer_size(), win.outer_position()) {
                    let sf = win.scale_factor().unwrap_or(1.0);
                    let min_w_phys = (650f64 * sf) as u32;
                    let min_h_phys = (600f64 * sf) as u32;
                    let state = app_handle.state::<AppState>();

                    if size.width >= min_w_phys && size.height >= min_h_phys {
                        let monitors = win.available_monitors().unwrap_or_default();
                        let on_screen = monitors.iter().any(|m| {
                            let mp = m.position();
                            let ms = m.size();
                            pos.x < mp.x + ms.width as i32
                                && pos.x + size.width as i32 > mp.x
                                && pos.y < mp.y + ms.height as i32
                                && pos.y + size.height as i32 > mp.y
                        });
                        if on_screen {
                            *state.main_geometry.lock().unwrap() = Some((size, pos));
                        }
                    } else {
                        tracing::warn!("Window corrupted to {:?}, restoring last known size", size);
                        let saved = *state.main_geometry.lock().unwrap();
                        if let Some((saved_size, saved_pos)) = saved {
                            let _ = win.set_size(saved_size);
                            let _ = win.set_position(saved_pos);
                        } else {
                            let _ = win.set_size(tauri::PhysicalSize::new(min_w_phys, min_h_phys));
                            let _ = win.center();
                        }
                    }
                }
            }
            tauri::WindowEvent::Moved(pos) => {
                if win.is_minimized().unwrap_or(false) {
                    return;
                }
                let state = app_handle.state::<AppState>();
                let mut g = state.main_geometry.lock().unwrap();
                if let Some((size, _)) = *g {
                    *g = Some((size, *pos));
                }
            }
            _ => {}
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        let session = MediaSession::new(std::ptr::null_mut(), app.clone());
        if session.is_some() {
            *app.state::<AppState>().media_session.lock().unwrap() = session;
        }
    }

    let win_clone = window.clone();

    // macOS: Use native NSWindowWillMiniaturizeNotification instead of polling
    #[cfg(target_os = "macos")]
    {
        let app_handle = app.clone();
        // Small delay to ensure the window is fully initialized
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            crate::platform::macos::register_miniaturize_observer(&app_handle);
            crate::platform::macos::register_default_device_listener(app_handle);
            tracing::info!("macOS observers registered");
        });
    }

    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Resized(_) => {
            #[cfg(not(target_os = "macos"))]
            {
                let win = win_clone.clone();
                let is_min = win.is_minimized().unwrap_or(false);

                if is_min {
                    let state = win.state::<crate::state::AppState>();
                    let minimize_to_tray = state.inner.lock().unwrap().minimize_to_tray;

                    if minimize_to_tray {
                        tracing::info!("Window minimized, hiding to tray");
                        let _ = win.hide();
                    }
                }
            }
            crate::commands::internal_layout_link_view(&win_clone);
        }
        tauri::WindowEvent::Focused(_focused) => {
            #[cfg(target_os = "macos")]
            if *_focused && win_clone.is_visible().unwrap_or(false) {
                tracing::info!("Window focused, setting Regular policy");
                let _ = win_clone.app_handle().set_activation_policy(tauri::ActivationPolicy::Regular);
            }
        }
        tauri::WindowEvent::ScaleFactorChanged { .. } => {
            crate::commands::internal_layout_link_view(&win_clone);
        }
        tauri::WindowEvent::CloseRequested { api, .. } => {
            let state = win_clone.state::<crate::state::AppState>();
            let close_to_tray = state.inner.lock().unwrap().close_to_tray;

            if close_to_tray {
                tracing::info!("DEBUG: Close requested, hiding to tray");
                #[cfg(target_os = "macos")]
                let _ = win_clone.app_handle().set_activation_policy(tauri::ActivationPolicy::Accessory);
                let _ = win_clone.hide();
                api.prevent_close();
            } else {
                let _ = win_clone.hide();
                let handle = win_clone.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    for win in handle.webview_windows().values() {
                        let _ = win.destroy();
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                    crate::IS_QUITTING.store(true, std::sync::atomic::Ordering::SeqCst);
                    handle.exit(0);
                });
                api.prevent_close();
            }
        }
        _ => {}
    });
}

/// Initialises OS media transport controls for the "main" window at startup.
pub fn setup_os_media_controls(app: &App) {
    if let Some(window) = app.get_webview_window("main") {
        attach_main_window_listeners(&app.handle().clone(), &window);
    }
}

/// Returns the "main" window, recreating it (with all listeners/SMTC/thumbbar reattached) if it
/// was destroyed by the idle-destroy poller. Reused by every "show the main window" call site.
pub fn get_or_create_main_window(app: &tauri::AppHandle) -> tauri::WebviewWindow {
    if let Some(window) = app.get_webview_window("main") {
        return window;
    }

    tracing::info!("Recreating destroyed 'main' window");

    // Show loading indicator while WebView reloads
    let splash = platform::splash::SplashScreen::show(); // Windows: native Win32 splash
    #[cfg(not(target_os = "windows"))]
    show_html_splash_handle(app); // macOS/Linux: HTML splash window

    let saved_geometry = *app.state::<AppState>().main_geometry.lock().unwrap();

    let mut builder = tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("Radiocove")
    .inner_size(1100.0, 700.0)
    .min_inner_size(650.0, 600.0)
    .resizable(true)
    .decorations(false)
    .visible(false);

    if let Some((size, pos)) = saved_geometry {
        builder = builder
            .inner_size(size.width as f64, size.height as f64)
            .position(pos.x as f64, pos.y as f64);
    } else {
        builder = builder.center();
    }

    match builder.build() {
        Ok(window) => {
            attach_main_window_listeners(app, &window);
            await_frontend_and_close_splash(app.clone(), splash);
            window
        }
        Err(e) => {
            panic!("Failed to recreate 'main' window: {:?}", e);
        }
    }
}

/// Polls until the frontend window appears, then hides all splash screens.
pub fn await_frontend_and_close_splash(
    app_handle: tauri::AppHandle,
    splash_handle: Option<platform::splash::SplashScreen>,
) {
    let poll_win = app_handle.get_webview_window("main");
    if poll_win.is_none() {
        if let Some(s) = splash_handle {
            s.close();
        }
        return;
    }
    let poll_win = poll_win.unwrap();

    std::thread::spawn(move || {
        let show_and_close_splash = |handle: &tauri::AppHandle| {
            // Ensure window is within visible screen bounds (fixes off-screen issue on Windows)
            #[cfg(target_os = "windows")]
            {
                if let (Ok(pos), Ok(size)) = (poll_win.outer_position(), poll_win.outer_size()) {
                    let monitors = poll_win.available_monitors().unwrap_or_default();
                    let on_screen = monitors.iter().any(|m| {
                        let mp = m.position();
                        let ms = m.size();
                        pos.x < mp.x + ms.width as i32
                            && pos.x + size.width as i32 > mp.x
                            && pos.y < mp.y + ms.height as i32
                            && pos.y + size.height as i32 > mp.y
                    });
                    if !on_screen {
                        tracing::warn!("Window is off-screen, centering");
                        let _ = poll_win.center();
                    }
                }
            }
            // Show main window and grab focus FIRST
            let _ = poll_win.show();
            let _ = poll_win.set_focus();
            // Small delay to let the window actually appear before closing splash
            std::thread::sleep(std::time::Duration::from_millis(50));
            // THEN close splashes
            if let Some(s) = splash_handle {
                s.close();
            }
            if let Some(sw) = handle.get_webview_window("splash") {
                let _ = sw.close();
            }
        };

        // Wait for frontend to become visible or timeout after 10s
        for i in 0..100 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if poll_win.is_visible().unwrap_or(false) {
                tracing::info!("Frontend ready after {}ms", i * 100);
                show_and_close_splash(&app_handle);
                return;
            }
        }
        tracing::warn!("Frontend didn't show window within 10s, forcing show");
        show_and_close_splash(&app_handle);
    });
}

use include_dir::{include_dir, Dir};
static LOCALES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../src/locales");

fn get_tray_text(lang: &str, key: &str) -> String {
    let file_name = format!("{}.json", lang);
    let file = LOCALES_DIR.get_file(&file_name)
        .or_else(|| LOCALES_DIR.get_file("en.json"));
    
    if let Ok(json_str) = file.and_then(|f| f.contents_utf8()).ok_or(()) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
            if let Some(text) = v["tray"][key].as_str() {
                return text.to_string();
            }
        }
    }
    key.to_string()
}

fn update_tray_menu(app: &tauri::AppHandle, lang: &str) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem, Submenu, PredefinedMenuItem};
    use crate::player::types::PlaybackStatus;

    let state = app.state::<crate::state::AppState>();
    let (status, station_name, song_title) = {
        let inner = state.inner.lock().unwrap();
        let s_name = inner.station_name.clone().unwrap_or_else(|| "Radiocove".to_string());
        let s_title = inner.stream_metadata.as_ref().and_then(|m| m.title.clone());
        (inner.status.clone(), s_name, s_title)
    };

    let info_text = if let Some(t) = &song_title {
        if t.trim().is_empty() {
            format!("📻 {}", station_name)
        } else {
            let mut display = format!("{} - {}", station_name, t);
            if display.chars().count() > 36 {
                display = display.chars().take(33).collect::<String>() + "...";
            }
            format!("📻 {}", display)
        }
    } else if station_name != "Radiocove" && !station_name.trim().is_empty() {
        format!("📻 {}", station_name)
    } else {
        "Radiocove".to_string()
    };

    let info_i = MenuItem::with_id(app, "info", info_text, false, None::<&str>)?;
    
    let status_text = match status {
        PlaybackStatus::Playing | PlaybackStatus::Connecting => format!("✨ {}", get_tray_text(lang, "playing")),
        PlaybackStatus::Paused => format!("⏸ {}", get_tray_text(lang, "paused")),
        _ => format!("⏹ {}", get_tray_text(lang, "stopped")),
    };
    let status_i = MenuItem::with_id(app, "status", status_text, false, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;

    let play_label = match status {
        PlaybackStatus::Playing | PlaybackStatus::Connecting => format!("⏸ {}", get_tray_text(lang, "pause")),
        _ => format!("▶ {}", get_tray_text(lang, "play")),
    };
    let play_pause_i = MenuItem::with_id(app, "play_pause", play_label, true, None::<&str>)?;
    
    let next_i = MenuItem::with_id(app, "next", format!("⏭ {}", get_tray_text(lang, "next")), true, None::<&str>)?;
    let prev_i = MenuItem::with_id(app, "prev", format!("⏮ {}", get_tray_text(lang, "prev")), true, None::<&str>)?;
    
    let vol_m = Submenu::with_id(app, "vol_menu", format!("🔊 {}", get_tray_text(lang, "volume")), true)?;
    let v0 = MenuItem::with_id(app, "vol_0", get_tray_text(lang, "mute"), true, None::<&str>)?;
    let v20 = MenuItem::with_id(app, "vol_20", "20%", true, None::<&str>)?;
    let v50 = MenuItem::with_id(app, "vol_50", "50%", true, None::<&str>)?;
    let v80 = MenuItem::with_id(app, "vol_80", "80%", true, None::<&str>)?;
    let v100 = MenuItem::with_id(app, "vol_100", "100%", true, None::<&str>)?;
    let _ = vol_m.append_items(&[&v0, &v20, &v50, &v80, &v100])?;
    
    let sep2 = PredefinedMenuItem::separator(app)?;

    let quit_i = MenuItem::with_id(app, "quit", get_tray_text(lang, "quit"), true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", get_tray_text(lang, "mainWindow"), true, None::<&str>)?;
    let mini_i = MenuItem::with_id(app, "mini", get_tray_text(lang, "miniPlayer"), true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[
        &info_i, &status_i, &sep1, 
        &play_pause_i, &next_i, &prev_i, 
        &vol_m, &sep2, 
        &mini_i, &show_i, &quit_i
    ])?;

    if let Some(tray) = app.tray_by_id("main_tray") {
        let _ = tray.set_menu(Some(menu));
    }
    
    Ok(())
}

/// Builds the "tray" mini-player window and attaches its focus-loss-hide listener
/// (Linux/macOS). Called at startup, and again to recreate the window after the
/// idle-destroy poller destroys it.
pub fn create_tray_window(app: &tauri::AppHandle) -> tauri::Result<tauri::WebviewWindow> {
    if let Some(state) = app.try_state::<AppState>() {
        state.tray_ready.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    #[allow(unused_mut)]
    let mut builder = tauri::WebviewWindowBuilder::new(
        app,
        "tray",
        tauri::WebviewUrl::App("tray.html".into()),
    )
    .title("Radiocove Mini Player")
    .inner_size(320.0, 88.0)
    .decorations(false)
    .always_on_top(true)
    .resizable(false)
    .visible(false);

    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.transparent(true).skip_taskbar(true);
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, transparent() requires macos-private-api feature or tauri.conf.json configuration.
        // skip_taskbar(true) is also unsupported on macOS via WebviewWindowBuilder.
    }

    let _tray_win = builder.build()?;

    // Clone for event handler (only used on Linux/macOS)
    #[cfg(not(target_os = "windows"))]
    let tray_win_clone = _tray_win.clone();

    // Hide tray window when it loses focus (Linux/macOS)
    #[cfg(not(target_os = "windows"))]
    _tray_win.on_window_event(move |event| match event {
        tauri::WindowEvent::Focused(focused) => {
            if !*focused {
                #[cfg(target_os = "linux")]
                {
                    tracing::info!("Tray Window: Lost focus, waiting before hiding...");
                    let w = tray_win_clone.clone();
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                        // Avoid hiding if it quickly regained focus
                        if !w.is_focused().unwrap_or(false) && w.is_visible().unwrap_or(false) {
                            tracing::info!("Tray Window: Still unfocused, hiding.");
                            let _ = w.hide();
                        }
                    });
                }
                #[cfg(target_os = "macos")]
                {
                    let _ = tray_win_clone.hide();
                }
            } else {
                #[cfg(target_os = "linux")]
                tracing::info!("Tray Window: Gained focus");
            }
        }
        _ => {}
    });

    Ok(_tray_win)
}

/// Returns the "tray" window, recreating it if it was destroyed by the idle-destroy poller.
pub fn get_or_create_tray_window(app: &tauri::AppHandle) -> Option<tauri::WebviewWindow> {
    if let Some(window) = app.get_webview_window("tray") {
        return Some(window);
    }
    tracing::info!("Recreating destroyed 'tray' window");
    match create_tray_window(app) {
        Ok(window) => Some(window),
        Err(e) => {
            tracing::error!("Failed to recreate 'tray' window: {:?}", e);
            None
        }
    }
}

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    use tauri::{Emitter, Manager};
    #[cfg(not(target_os = "linux"))]
    use tauri_plugin_positioner::{WindowExt, Position};

    let dir = app.path().app_data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let settings = crate::settings::Settings::load(&dir);
    let initial_lang = settings.language.unwrap_or_else(|| "en".to_string());
    let lang_ref = std::sync::Arc::new(std::sync::Mutex::new(initial_lang.clone()));

    create_tray_window(&app.handle().clone())?;
    // Pre-warmed hidden at startup so the first tray click opens instantly. But `ever_shown`
    // starting false means the idle-destroy poller would never reap it until the user opens
    // it for real once — seed the tracker now so it's still destroyed after TRAY_GRACE if the
    // user never opens the tray at all.
    {
        let state = app.state::<AppState>();
        let mut t = state.tray_idle.lock().unwrap();
        t.ever_shown = true;
        t.hidden_since = Some(std::time::Instant::now());
    }

    let _tray = TrayIconBuilder::with_id("main_tray")
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(false)
        .tooltip("Radiocove")
        .on_menu_event(move |app: &tauri::AppHandle, event| {
            match event.id.as_ref() {
                "mini" => {
                    println!("TRAY: 'mini' menu item clicked");
                    tracing::info!("Tray: 'mini' menu item clicked");
                    if let Some(window) = get_or_create_tray_window(app) {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            #[cfg(target_os = "linux")]
                            {
                                if let Ok(cursor_pos) = app.cursor_position() {
                                    let size = window.inner_size().unwrap_or(tauri::PhysicalSize { width: 320, height: 88 });
                                    let monitor = app.monitor_from_point(cursor_pos.x, cursor_pos.y).ok().flatten()
                                        .or_else(|| window.primary_monitor().ok().flatten());
                                    
                                    if let Some(m) = monitor {
                                        let m_pos = m.position();
                                        let m_size = m.size();
                                        let rel_x = cursor_pos.x - m_pos.x as f64;
                                        let rel_y = cursor_pos.y - m_pos.y as f64;
                                        let is_top = rel_y < (m_size.height as f64 / 2.0);
                                        let is_right = rel_x > (m_size.width as f64 / 2.0);
                                        
                                        let margin_x = 12;
                                        let margin_y = 36;
                                        
                                        let x = if is_right {
                                            m_pos.x + m_size.width as i32 - size.width as i32 - margin_x
                                        } else {
                                            m_pos.x + margin_x
                                        };
                                        let y = if is_top {
                                            m_pos.y + margin_y
                                        } else {
                                            m_pos.y + (m_size.height as i32) - (size.height as i32) - margin_y
                                        };
                                        
                                        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                                    }
                                }
                            }
                            #[cfg(not(target_os = "linux"))]
                            let _ = window.move_window(Position::TrayCenter);
                            
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                            
                            // Extra focus-force for Linux to ensure hide-on-blur works
                            #[cfg(target_os = "linux")]
                            {
                                let w = window.clone();
                                tauri::async_runtime::spawn(async move {
                                    for delay in [50, 150, 300, 600] {
                                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                                        if !w.is_visible().unwrap_or(false) { break; }
                                        if w.is_focused().unwrap_or(false) { break; }
                                        
                                        let _ = w.unminimize();
                                        let _ = w.set_focus();
                                        let _ = w.set_always_on_top(false);
                                        let _ = w.set_always_on_top(true);
                                    }
                                });
                            }
                            
                            let _ = window.emit("tray-opened", ());
                        }
                    }
                }
                "quit" => {
                    println!("TRAY: 'quit' menu item clicked");
                    let h = app.clone();
                    
                    tauri::async_runtime::spawn(async move {
                        // Destroy ALL windows (Main, Tray, Browser views etc.)
                        for win in h.webview_windows().values() {
                            let _ = win.destroy();
                        }
                        
                        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                        crate::IS_QUITTING.store(true, std::sync::atomic::Ordering::SeqCst);
                        h.exit(0);
                    });
                }
                "show" => {
                    println!("TRAY: 'show' menu item clicked");
                    // This handler runs on the main thread; get_or_create_main_window's
                    // WebviewWindowBuilder::build() needs the main loop to process the
                    // creation, so calling it inline here would deadlock. Run on its own thread.
                    let app = app.clone();
                    std::thread::spawn(move || {
                        let window = get_or_create_main_window(&app);
                        #[cfg(target_os = "macos")]
                        {
                            let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                        }
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    });
                }
                "play_pause" => {
                    let h = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = h.emit("media-key", "toggle");
                    });
                }
                "next" => {
                    let h = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = h.emit("media-key", "next");
                    });
                }
                "prev" => {
                    let h = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = h.emit("media-key", "previous");
                    });
                }
                "vol_0" => { 
                    let h = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::commands::set_volume(0.0, h.clone(), h.state()).await;
                    });
                }
                "vol_20" => { 
                    let h = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::commands::set_volume(0.2, h.clone(), h.state()).await;
                    });
                }
                "vol_50" => { 
                    let h = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::commands::set_volume(0.5, h.clone(), h.state()).await;
                    });
                }
                "vol_80" => { 
                    let h = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::commands::set_volume(0.8, h.clone(), h.state()).await;
                    });
                }
                "vol_100" => { 
                    let h = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::commands::set_volume(1.0, h.clone(), h.state()).await;
                    });
                }

                _ => {}
            }
        })
        .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| {
            #[cfg(target_os = "linux")]
            println!("TRAY: Icon Event: {:?}", event);
            
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            
            match event {
                TrayIconEvent::Click {
                    button,
                    button_state,
                    position,
                    ..
                } if button == MouseButton::Left && (cfg!(target_os = "linux") || button_state == MouseButtonState::Up) => {
                    let _ = position;
                    #[cfg(target_os = "windows")]
                    {
                        let last_hide = crate::platform::mouse_hook::LAST_HIDE_TIME.load(std::sync::atomic::Ordering::SeqCst);
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or(std::time::Duration::from_millis(0))
                            .as_millis() as u64;
                        
                        // If the window was hidden less than 200ms ago by the global mouse hook, 
                        // this click event is the same tray icon click finishing its MouseUp phase.
                        if now > 0 && now.saturating_sub(last_hide) < 200 {
                            return;
                        }
                    }

                    // get_or_create_tray_window's WebviewWindowBuilder::build() needs the main
                    // loop to process the creation; doing this inline here (this handler runs
                    // on the main thread) blocks that loop for as long as the OS takes to spin
                    // up the webview controller. Run the whole thing off-thread.
                    let app = tray.app_handle().clone();
                    std::thread::spawn(move || {
                    if let Some(window) = get_or_create_tray_window(&app) {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            #[cfg(target_os = "linux")]
                            {
                                // Screen-aware positioning for Linux tray clicks
                                let size = window.inner_size().unwrap_or(tauri::PhysicalSize { width: 320, height: 88 });

                                // Identify which monitor the click happened on
                                let monitor = app.monitor_from_point(position.x, position.y).ok().flatten()
                                    .or_else(|| window.primary_monitor().ok().flatten());

                                if let Some(m) = monitor {
                                    let m_pos = m.position();
                                    let m_size = m.size();
                                    let rel_x = position.x - m_pos.x as f64;
                                    let rel_y = position.y - m_pos.y as f64;
                                    let is_top = rel_y < (m_size.height as f64 / 2.0);
                                    let is_right = rel_x > (m_size.width as f64 / 2.0);

                                    let margin_x = 12;
                                    let margin_y = 36;

                                    let x = if is_right {
                                        m_pos.x + m_size.width as i32 - size.width as i32 - margin_x
                                    } else {
                                        m_pos.x + margin_x
                                    };
                                    let y = if is_top {
                                        m_pos.y + margin_y
                                    } else {
                                        m_pos.y + (m_size.height as i32) - (size.height as i32) - margin_y
                                    };

                                    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
                                }
                            }
                            #[cfg(not(target_os = "linux"))]
                            let _ = window.move_window(Position::TrayCenter);

                            // Wait briefly for the frontend to actually paint (see
                            // `mark_tray_ready`) before showing, so a freshly (re)created
                            // window doesn't flash blank/transparent first. Already-warm
                            // windows have this set from their previous open already, so this
                            // is a no-op for the common case.
                            if let Some(state) = app.try_state::<AppState>() {
                                for _ in 0..20 {
                                    if state.tray_ready.load(std::sync::atomic::Ordering::SeqCst) {
                                        break;
                                    }
                                    std::thread::sleep(std::time::Duration::from_millis(25));
                                }
                            }

                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.emit("tray-opened", ());

                            #[cfg(target_os = "windows")]
                            {
                                let hwnd_val = if let Ok(hwnd) = window.hwnd() {
                                    hwnd.0 as isize
                                } else {
                                    0
                                };

                                // 1. Give focus best-effort
                                if hwnd_val != 0 {
                                    tauri::async_runtime::spawn(async move {
                                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                                        extern "system" { fn SetForegroundWindow(hwnd: isize) -> i32; }
                                        unsafe { SetForegroundWindow(hwnd_val); }
                                    });
                                }

                                // 2. The ONLY bulletproof way on Windows for frameless tray apps:
                                // Use a global mouse hook (WH_MOUSE_LL) to detect left/right clicks
                                // that happen outside of our window.
                                let app_handle = window.app_handle().clone();
                                tauri::async_runtime::spawn(async move {
                                    // A small delay to avoid catching the initial tray icon click
                                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                                    // Start the hook logic on a dedicated background thread since
                                    // Windows hooks require a message loop or need to block.
                                    std::thread::spawn(move || {
                                        crate::platform::mouse_hook::start_mouse_hook(app_handle, hwnd_val);
                                    });
                                });
                            }
                            #[cfg(not(target_os = "windows"))]
                            {
                                let _ = window.set_focus();
                                #[cfg(target_os = "linux")]
                                {
                                    let w = window.clone();
                                    tauri::async_runtime::spawn(async move {
                                        for delay in [50, 150, 300, 600] {
                                            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                                            if !w.is_visible().unwrap_or(false) { break; }
                                            if w.is_focused().unwrap_or(false) { break; }

                                            let _ = w.unminimize();
                                            let _ = w.set_focus();
                                            let _ = w.set_always_on_top(false);
                                            let _ = w.set_always_on_top(true);
                                        }
                                    });
                                }
                            }
                        }
                    }
                    });
                }
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => {
                    {
                        // Runs on the main thread; get_or_create_main_window's build() needs the
                        // main loop to process the creation, so do this off-thread to avoid a deadlock.
                        let app = tray.app_handle().clone();
                        std::thread::spawn(move || {
                        let main = get_or_create_main_window(&app);
                        let is_visible = main.is_visible().unwrap_or(false);
                        let is_minimized = main.is_minimized().unwrap_or(false);

                        if is_visible && !is_minimized {
                            let _ = main.hide();
                            #[cfg(target_os = "macos")]
                            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                        } else {
                            #[cfg(target_os = "macos")]
                            {
                                tracing::info!("Tray double-click: showing window, setting Regular policy");
                                let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                            }
                            if is_minimized {
                                let _ = main.unminimize();
                            }
                            let _ = main.show();
                            let _ = main.set_focus();
                        }
                        });
                    }
                }
                _ => {}
            }
        })
        .build(app)?;

    // Initialize menu
    let _ = update_tray_menu(app.handle(), &initial_lang);

    // Dynamic tray menu logic
    {
        use tauri::Listener;
        let lang_ref_listen = lang_ref.clone();
        let app_handle = app.handle().clone();
        let app_handle_inner = app_handle.clone();
        app_handle.listen("playback-status", move |_| {
            let current_lang = lang_ref_listen.lock().unwrap().clone();
            let _ = update_tray_menu(&app_handle_inner, &current_lang);
        });

        // Listen for language changes
        let lang_ref_inner = lang_ref.clone();
        let app_handle_lang = app.handle().clone();
        let app_handle_lang_inner = app_handle_lang.clone();
        app_handle_lang.listen("language-changed", move |event: tauri::Event| {
            if let Ok(new_lang) = serde_json::from_str::<String>(event.payload()) {
                let mut lang = lang_ref_inner.lock().unwrap();
                *lang = new_lang.clone();
                let _ = update_tray_menu(&app_handle_lang_inner, &new_lang);
            }
        });

        // Listen for metadata updates
        let lang_ref_meta = lang_ref.clone();
        let app_handle_meta = app.handle().clone();
        let app_handle_meta_inner = app_handle_meta.clone();
        app_handle_meta.listen("stream-metadata", move |_| {
            let current_lang = lang_ref_meta.lock().unwrap().clone();
            let _ = update_tray_menu(&app_handle_meta_inner, &current_lang);
        });
    }

    spawn_idle_window_destroyer(app.handle().clone());

    Ok(())
}

/// Destroys hidden "main"/"tray" windows after their grace period to free memory.
/// `ever_shown` prevents destroying "tray" before it's been opened even once.
/// The app keeps running with zero windows open (see RunEvent::ExitRequested in lib.rs).
/// Both the on/off switch and the grace period are user-configurable (Settings → Advanced)
/// and read live from `AppState` every tick, see `commands::save_window_idle_settings`.
fn spawn_idle_window_destroyer(app: tauri::AppHandle) {
    use std::sync::atomic::Ordering;

    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            let Some(state) = app.try_state::<AppState>() else { continue };

            if state.main_idle_destroy_enabled.load(Ordering::Relaxed) {
                let grace = std::time::Duration::from_secs(state.main_idle_grace_secs.load(Ordering::Relaxed) as u64);
                check_idle_window(&app, "main", &state.main_idle, grace);
            }
            if state.tray_idle_destroy_enabled.load(Ordering::Relaxed) {
                let grace = std::time::Duration::from_secs(state.tray_idle_grace_secs.load(Ordering::Relaxed) as u64);
                check_idle_window(&app, "tray", &state.tray_idle, grace);
            }
        }
    });
}

/// Drives auto-identify entirely in the backend so it keeps working with no window open
/// (it used to live in a React effect in App.jsx, which died whenever that component
/// unmounted — e.g. once the idle-destroy poller above tears down "main").
pub fn spawn_auto_identify_loop(app: tauri::AppHandle) {
    use tauri::Emitter;
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let Some(state) = app.try_state::<AppState>() else { continue };

            let (enabled, cooldown_success, cooldown_fail, is_playing) = {
                let ps = state.inner.lock().unwrap();
                (
                    ps.auto_identify,
                    ps.auto_identify_cooldown_success,
                    ps.auto_identify_cooldown_fail,
                    ps.status == crate::player::types::PlaybackStatus::Playing,
                )
            };

            if !enabled || !is_playing {
                continue;
            }

            let cooldown_secs = match *state.last_identify_status.lock().unwrap() {
                crate::state::IdentifyOutcome::Success => cooldown_success,
                crate::state::IdentifyOutcome::Fail => cooldown_fail,
            };
            let due = match *state.last_identify_attempt.lock().unwrap() {
                None => true,
                Some(t) => t.elapsed() >= std::time::Duration::from_secs(cooldown_secs as u64),
            };
            if !due {
                continue;
            }

            let Some(state) = app.try_state::<AppState>() else { continue };
            let result = crate::commands::identify_song(app.clone(), state).await;

            let Some(state) = app.try_state::<AppState>() else { continue };
            match result {
                Ok(Some(found)) => {
                    // A manual "Identify Now" click is already in flight; don't count this
                    // tick as our attempt, just retry next tick once it's done.
                    if found.get("_error_type").and_then(|v| v.as_str()) == Some("already_running") {
                        continue;
                    }

                    *state.last_identify_attempt.lock().unwrap() = Some(std::time::Instant::now());

                    if found.get("_error").is_some() {
                        *state.last_identify_status.lock().unwrap() = crate::state::IdentifyOutcome::Fail;
                        let _ = app.emit("auto-identify-result", serde_json::json!({ "ok": false }));
                    } else {
                        *state.last_identify_status.lock().unwrap() = crate::state::IdentifyOutcome::Success;

                        let station_name = {
                            let ps = state.inner.lock().unwrap();
                            ps.station_name.clone().unwrap_or_else(|| "Unknown Radio".to_string())
                        };
                        let song_link = found.get("song_link").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let song_record = serde_json::json!({
                            "title": found.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                            "artist": found.get("artist").and_then(|v| v.as_str()).unwrap_or(""),
                            "album": found.get("album").and_then(|v| v.as_str()).unwrap_or(""),
                            "release_date": found.get("release_date").and_then(|v| v.as_str()).unwrap_or(""),
                            "cover": found.get("cover").and_then(|v| v.as_str()).unwrap_or(""),
                            "song_link": song_link.clone(),
                            "station_name": station_name,
                            "found_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                            "source": "Shazam",
                            "sources": [{ "name": "Shazam", "link": song_link }],
                        });
                        let _ = crate::commands::save_identified_song(app.clone(), song_record);
                        let _ = app.emit("auto-identify-result", serde_json::json!({ "ok": true, "song": found }));
                    }
                }
                _ => {
                    *state.last_identify_attempt.lock().unwrap() = Some(std::time::Instant::now());
                    *state.last_identify_status.lock().unwrap() = crate::state::IdentifyOutcome::Fail;
                }
            }
        }
    });
}

fn check_idle_window(
    app: &tauri::AppHandle,
    label: &str,
    tracker: &std::sync::Mutex<crate::state::WindowIdleTracker>,
    grace: std::time::Duration,
) {
    let Some(window) = app.get_webview_window(label) else { return };
    let visible = window.is_visible().unwrap_or(true);
    let mut t = tracker.lock().unwrap();

    if visible {
        t.ever_shown = true;
        t.hidden_since = None;
        return;
    }

    if !t.ever_shown {
        return;
    }

    match t.hidden_since {
        None => t.hidden_since = Some(std::time::Instant::now()),
        Some(since) if since.elapsed() >= grace => {
            tracing::info!("'{}' window idle for {:?}, destroying to free memory", label, grace);
            let _ = window.destroy();
            *t = crate::state::WindowIdleTracker::default();
        }
        Some(_) => {}
    }
}
