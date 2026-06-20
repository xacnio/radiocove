use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;

use crate::player::types::{PlaybackStatus, StreamMetadata};
use crate::player::PlayerHandle;
use crate::services::discord_rpc::DiscordRpc;
use crate::services::media::MediaSession;

/// Tracks whether a window has ever been shown, and if currently hidden, since when —
/// used by the idle-destroy poller to free background windows without touching every
/// individual hide call site.
#[derive(Default)]
pub struct WindowIdleTracker {
    pub ever_shown: bool,
    pub hidden_since: Option<Instant>,
}

/// Outcome of the last auto-identify attempt, used to pick which cooldown applies next.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum IdentifyOutcome {
    #[default]
    Success,
    Fail,
}

pub struct AppState {
    pub inner: Arc<Mutex<PlayerState>>,
    pub proxy_port: u16,
    /// Async mutex to serialize play() calls and prevent race conditions
    pub play_lock: tokio::sync::Mutex<()>,
    /// Maps original master playlist URL -> last known valid variant URL (with session ID)
    pub hls_session_cache: Arc<Mutex<HashMap<String, String>>>,
    /// Last time device change restart was triggered (for debouncing)
    pub last_device_restart: Arc<Mutex<Option<Instant>>>,
    /// Discord Rich Presence client
    pub discord_rpc: Arc<DiscordRpc>,
    /// OS media transport controls, rebuilt whenever the "main" window is recreated
    /// (Windows SMTC is bound to a HWND and can't be rebound to a new one).
    pub media_session: Mutex<Option<MediaSession>>,
    pub main_idle: Mutex<WindowIdleTracker>,
    pub tray_idle: Mutex<WindowIdleTracker>,
    /// Last known good "main" window size/position, preserved across destroy/recreate
    /// cycles so the window reopens where the user left it.
    pub main_geometry: Mutex<Option<(tauri::PhysicalSize<u32>, tauri::PhysicalPosition<i32>)>>,
    /// Set by the tray frontend once it has actually painted (see `mark_tray_ready`). Reset
    /// to false whenever the "tray" window is (re)built, so the tray-icon click handler can
    /// wait for real content instead of showing a blank/transparent window first.
    pub tray_ready: AtomicBool,
    /// When the auto-identify background loop last actually ran an attempt (manual
    /// "Identify Now" clicks don't touch this). `None` means "never — go right away".
    pub last_identify_attempt: Mutex<Option<Instant>>,
    /// Outcome of that last attempt, used to pick the success/fail cooldown for the next one.
    pub last_identify_status: Mutex<IdentifyOutcome>,
    /// Live copies of the idle-destroy settings (see `setup::spawn_idle_window_destroyer`),
    /// so toggling them in Settings takes effect immediately without a restart.
    pub main_idle_destroy_enabled: AtomicBool,
    pub main_idle_grace_secs: AtomicU32,
    pub tray_idle_destroy_enabled: AtomicBool,
    pub tray_idle_grace_secs: AtomicU32,
}

pub struct PlayerState {
    pub status: PlaybackStatus,
    pub current_url: Option<String>,
    pub station_name: Option<String>,
    pub station_image: Option<String>, // Local file:/// URL for display
    pub default_cover: Option<String>,
    pub enriched_cover: Option<String>, // iTunes/enriched cover for Discord
    pub enriched_album: Option<String>, // Album name from iTunes
    pub volume: f32,
    pub stream_metadata: Option<StreamMetadata>,
    pub handle: Option<PlayerHandle>,
    pub preview_handle: Option<PlayerHandle>,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub output_device: Option<String>,
    pub skip_ads: bool,
    pub auto_identify: bool,
    pub auto_identify_cooldown_success: u32,
    pub auto_identify_cooldown_fail: u32,
}

impl PlayerState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        volume: f32,
        last_url: Option<String>,
        minimize_to_tray: bool,
        close_to_tray: bool,
        output_device: Option<String>,
        skip_ads: bool,
        auto_identify: bool,
        auto_identify_cooldown_success: u32,
        auto_identify_cooldown_fail: u32,
    ) -> Self {
        Self {
            status: PlaybackStatus::Stopped,
            current_url: last_url,
            station_name: None,
            station_image: None,
            default_cover: None,
            enriched_cover: None,
            enriched_album: None,
            volume,
            stream_metadata: None,
            handle: None,
            preview_handle: None,
            minimize_to_tray,
            close_to_tray,
            output_device,
            skip_ads,
            auto_identify,
            auto_identify_cooldown_success,
            auto_identify_cooldown_fail,
        }
    }
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        volume: f32,
        last_url: Option<String>,
        minimize_to_tray: bool,
        close_to_tray: bool,
        output_device: Option<String>,
        skip_ads: bool,
        discord_enabled: bool,
        auto_identify: bool,
        auto_identify_cooldown_success: u32,
        auto_identify_cooldown_fail: u32,
        main_idle_destroy_enabled: bool,
        main_idle_grace_secs: u32,
        tray_idle_destroy_enabled: bool,
        tray_idle_grace_secs: u32,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PlayerState::new(
                volume,
                last_url,
                minimize_to_tray,
                close_to_tray,
                output_device,
                skip_ads,
                auto_identify,
                auto_identify_cooldown_success,
                auto_identify_cooldown_fail,
            ))),
            proxy_port: 0,
            play_lock: tokio::sync::Mutex::new(()),
            hls_session_cache: Arc::new(Mutex::new(HashMap::new())),
            last_device_restart: Arc::new(Mutex::new(None)),
            discord_rpc: Arc::new(DiscordRpc::new(discord_enabled)),
            media_session: Mutex::new(None),
            main_idle: Mutex::new(WindowIdleTracker::default()),
            tray_idle: Mutex::new(WindowIdleTracker::default()),
            main_geometry: Mutex::new(None),
            tray_ready: AtomicBool::new(false),
            last_identify_attempt: Mutex::new(None),
            last_identify_status: Mutex::new(IdentifyOutcome::default()),
            main_idle_destroy_enabled: AtomicBool::new(main_idle_destroy_enabled),
            main_idle_grace_secs: AtomicU32::new(main_idle_grace_secs),
            tray_idle_destroy_enabled: AtomicBool::new(tray_idle_destroy_enabled),
            tray_idle_grace_secs: AtomicU32::new(tray_idle_grace_secs),
        }
    }
}
