//! Favicon / cover image commands: download, upload, batch cache, image search.

use tauri::{AppHandle, Emitter, Manager, WebviewBuilder, WebviewUrl};
use tracing::info;

use crate::error::AppError;

use super::{app_data_dir, path_to_file_url};

/// Download and cache a cover image, converting to PNG for SMTC compatibility.
/// Used internally by the player command and also exposed for batch operations.
pub(crate) async fn download_cover(url: String, app: AppHandle) -> Result<String, String> {
    if url.starts_with("file:///") {
        return Ok(url);
    }

    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;

    // Use a hash of the URL as filename — always save as PNG for SMTC compatibility
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();

    let file_path = cache_dir.join(format!("cover_{}.png", hash));

    // If already converted and valid PNG, reuse
    if file_path.exists() {
        if let Ok(header) = std::fs::read(&file_path) {
            if header.len() > 8 && header[..4] == [0x89, 0x50, 0x4E, 0x47] {
                return Ok(path_to_file_url(&file_path));
            }
        }
        let _ = std::fs::remove_file(&file_path);
    }

    let bytes = if url.starts_with("data:image/") {
        // Handle data URL (base64)
        if let Some(comma_pos) = url.find(',') {
            let base64_str = &url[comma_pos + 1..];
            use base64::{engine::general_purpose, Engine as _};
            general_purpose::STANDARD
                .decode(base64_str)
                .map_err(|e| format!("base64 decode failed: {}", e))?
        } else {
            return Err("invalid data url".into());
        }
    } else {
        // Download regular URL
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("download failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        resp.bytes()
            .await
            .map_err(|e| format!("read body failed: {}", e))?
            .to_vec()
    };

    if bytes.is_empty() {
        return Err("empty image data".into());
    }

    // Decode the image (supports png, jpg, webp, gif, ico) and re-encode as PNG
    let img = image::load_from_memory(&bytes).map_err(|e| format!("image decode failed: {}", e))?;

    img.save_with_format(&file_path, image::ImageFormat::Png)
        .map_err(|e| format!("png save failed: {}", e))?;

    info!(
        "download_cover: converted to PNG at {:?} (original {} bytes)",
        file_path,
        bytes.len()
    );

    let result = path_to_file_url(&file_path);

    info!("download_cover: returning {}", result);
    Ok(result)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FaviconEntry {
    pub uuid: String,
    pub url: String,
}

#[tauri::command]
pub async fn batch_cache_favicons(
    entries: Vec<FaviconEntry>,
    app: AppHandle,
) -> Result<std::collections::HashMap<String, String>, AppError> {
    use futures_util::stream::{self, StreamExt};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    let total = entries.len() as u32;
    let done = Arc::new(AtomicU32::new(0));

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| AppError::Connection(e.to_string()))?;
    std::fs::create_dir_all(&cache_dir).map_err(|e| AppError::Connection(e.to_string()))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    let results = stream::iter(entries)
        .map(|entry| {
            let client = client.clone();
            let cache_dir = cache_dir.clone();
            let done = done.clone();
            let app = app.clone();
            async move {
                let result = async {
                    if entry.url.is_empty() {
                        return None;
                    }
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    entry.url.hash(&mut hasher);
                    let hash = hasher.finish();
                    let file_path = cache_dir.join(format!("cover_{}.png", hash));

                    // Check cache
                    if file_path.exists() {
                        if let Ok(header) = std::fs::read(&file_path) {
                            if header.len() > 8 && header[..4] == [0x89, 0x50, 0x4E, 0x47] {
                                let p = path_to_file_url(&file_path);
                                return Some((entry.uuid.clone(), p));
                            }
                        }
                        let _ = std::fs::remove_file(&file_path);
                    }

                    let resp = client.get(&entry.url).send().await.ok()?;
                    if !resp.status().is_success() {
                        return None;
                    }
                    let bytes = resp.bytes().await.ok()?;
                    if bytes.is_empty() {
                        return None;
                    }

                    let img = image::load_from_memory(&bytes).ok()?;
                    let thumb = img.thumbnail(64, 64);
                    thumb
                        .save_with_format(&file_path, image::ImageFormat::Png)
                        .ok()?;

                    let p = path_to_file_url(&file_path);
                    Some((entry.uuid.clone(), p))
                }
                .await;

                // Emit progress
                let current = done.fetch_add(1, Ordering::Relaxed) + 1;
                let _ = app.emit(
                    "favicon-progress",
                    serde_json::json!({
                        "done": current,
                        "total": total,
                    }),
                );

                result
            }
        })
        .buffer_unordered(15)
        .collect::<Vec<_>>()
        .await;

    let map: std::collections::HashMap<String, String> = results.into_iter().flatten().collect();

    info!("batch_cache_favicons: cached {}/{}", map.len(), total);
    Ok(map)
}

#[tauri::command]
pub async fn upload_custom_favicon(
    app: AppHandle,
    bytes: Vec<u8>,
    ext: String,
) -> Result<String, AppError> {
    let dir = app_data_dir(&app)?;
    let p = dir.join(format!(
        "custom_{}.{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        ext
    ));
    std::fs::write(&p, bytes).map_err(|e| AppError::Settings(e.to_string()))?;
    Ok(format!(
        "file:///{}",
        p.to_string_lossy().replace("\\", "/")
    ))
}

#[tauri::command]
pub async fn download_custom_favicon(app: AppHandle, url: String) -> Result<String, AppError> {
    // Handle base64 data URIs directly (from Google Image search)
    if url.starts_with("data:image/") {
        // Parse: data:image/png;base64,iVBORw0KGgo...
        let ext = if url.starts_with("data:image/png") {
            "png"
        } else if url.starts_with("data:image/webp") {
            "webp"
        } else {
            "jpg"
        };

        let base64_data = url
            .find(",")
            .map(|pos| &url[pos + 1..])
            .ok_or_else(|| AppError::Settings("Invalid data URI".into()))?;

        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| AppError::Settings(format!("Base64 decode error: {}", e)))?;

        if bytes.len() < 100 {
            return Err(AppError::Settings("Image too small".into()));
        }

        let dir = app
            .path()
            .app_cache_dir()
            .map_err(|e| AppError::Settings(e.to_string()))?;
        std::fs::create_dir_all(&dir).map_err(|e| AppError::Settings(e.to_string()))?;
        let p = dir.join(format!(
            "custom_dl_{}.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            ext
        ));
        std::fs::write(&p, bytes).map_err(|e| AppError::Settings(e.to_string()))?;
        return Ok(format!(
            "file:///{}",
            p.to_string_lossy().replace("\\", "/")
        ));
    }

    // Regular URL download
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| AppError::Settings(e.to_string()))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Settings(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(AppError::Settings(format!(
            "Download failed: {}",
            resp.status()
        )));
    }

    // Guess extension
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let ext = if content_type.contains("png") {
        "png"
    } else if content_type.contains("webp") {
        "webp"
    } else if content_type.contains("gif") {
        "gif"
    } else if content_type.contains("svg") {
        "svg"
    } else {
        "jpg"
    };

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| AppError::Settings(e.to_string()))?;

    // Ignore truly tiny images (tracking pixels)
    if bytes.len() < 100 {
        return Err(AppError::Settings("Image too small".into()));
    }

    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| AppError::Settings(e.to_string()))?;
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Settings(e.to_string()))?;
    let p = dir.join(format!(
        "custom_dl_{}.{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        ext
    ));
    std::fs::write(&p, bytes).map_err(|e| AppError::Settings(e.to_string()))?;
    Ok(format!(
        "file:///{}",
        p.to_string_lossy().replace("\\", "/")
    ))
}

/// Searches Google Images via a hidden child webview (instead of raw HTTP requests).
///
/// Google's `udm=2` image search endpoint started rejecting plain `reqwest` calls with a
/// generic `emsg=SG_REL` error page — its anti-bot check fingerprints the TLS/HTTP stack,
/// which doesn't match a real browser no matter what headers/cookies are spoofed. Loading the
/// page in a real (hidden) WebView2/WebKit instance makes the request indistinguishable from
/// normal browsing, and lets us read the already-resolved `<img>` elements straight from the DOM.
#[tauri::command]
pub async fn search_images_internal(
    encoded_query: String,
    app: AppHandle,
) -> Result<Vec<String>, AppError> {
    info!("Searching images for: {}", encoded_query);

    let url = format!(
        "https://www.google.com/search?q={}&udm=2&imgar=s&hl=tr",
        encoded_query
    );
    let parsed_url = tauri::Url::parse(&url).map_err(|e| AppError::Settings(e.to_string()))?;

    let main_win = app
        .get_window("main")
        .ok_or_else(|| AppError::Settings("Main window not found".into()))?;

    let label = format!(
        "img-search-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let (tx, rx) = tokio::sync::oneshot::channel::<Vec<String>>();
    let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

    // Google's `dimg_*` thumbnails start out as 1x1 placeholder GIFs
    // (`data:image/gif;base64,R0lGODlh...`) and only get their real `data:image/jpeg` or
    // `https://*.gstatic.com/...` src assigned once they scroll into view (IntersectionObserver
    // lazy-load) — most of the grid is below the fold on first paint, so nothing loads until we
    // scroll. The length check filters out the placeholder (~80 chars; real thumbnails are 1000+).
    // `eval_with_callback` already JSON-serializes the JS expression's return value itself —
    // calling `JSON.stringify` here too would double-encode it (a JSON string containing JSON).
    // Google also renders small (~12-50px) "related searches" suggestion-chip thumbnails with the
    // same `dimg_*` id prefix — filter those out by size so only real result thumbnails come back.
    let extract_js = r#"
        Array.from(document.querySelectorAll('img[id^="dimg_"]'))
            .filter(img => {
                const w = parseInt(img.getAttribute('width') || img.naturalWidth || 0, 10);
                const h = parseInt(img.getAttribute('height') || img.naturalHeight || 0, 10);
                return w >= 100 && h >= 100;
            })
            .map(img => img.src)
            .filter(s => s && ((s.startsWith('data:image') && s.length > 1000) || s.startsWith('https://')))
            .slice(0, 30)
    "#.to_string();

    let label_for_load = label.clone();
    let app_for_load = app.clone();
    let tx_for_load = tx.clone();

    let outer_webview = main_win
        .add_child(
            WebviewBuilder::new(&label, WebviewUrl::External(parsed_url)).on_page_load(
                move |webview, payload| {
                    if payload.event() != tauri::webview::PageLoadEvent::Finished {
                        return;
                    }
                    let tx_inner = tx_for_load.clone();
                    let extract_js = extract_js.clone();
                    let app_inner = app_for_load.clone();
                    let label_inner = label_for_load.clone();
                    tauri::async_runtime::spawn(async move {
                        // Scroll down progressively to bring more of the lazy-loaded grid into
                        // view (most of it starts below the fold), then poll for newly-loaded
                        // images after each scroll step. Stop early once we have a decent batch.
                        let mut found: Vec<String> = Vec::new();
                        for attempt in 0..10u32 {
                            let delay = if attempt == 0 { 300 } else { 500 };
                            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;

                            let scroll_y = attempt * 1200;
                            let js = format!(
                                "window.scrollTo(0, {}); {}",
                                scroll_y, extract_js
                            );

                            let (attempt_tx, attempt_rx) = tokio::sync::oneshot::channel::<Vec<String>>();
                            let attempt_tx = std::sync::Mutex::new(Some(attempt_tx));
                            if webview
                                .eval_with_callback(js, move |result: String| {
                                    let images: Vec<String> = match serde_json::from_str(&result) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            tracing::warn!(
                                                "search_images_internal: eval result parse failed: {} | raw={}",
                                                e,
                                                &result[..result.len().min(300)]
                                            );
                                            Vec::new()
                                        }
                                    };
                                    if let Some(sender) = attempt_tx.lock().unwrap().take() {
                                        let _ = sender.send(images);
                                    }
                                })
                                .is_err()
                            {
                                continue;
                            }

                            if let Ok(Ok(images)) =
                                tokio::time::timeout(std::time::Duration::from_secs(2), attempt_rx).await
                            {
                                if images.len() >= 10 {
                                    found = images;
                                    break;
                                }
                                if images.len() > found.len() {
                                    found = images;
                                }
                            }
                        }

                        if let Some(sender) = tx_inner.lock().unwrap().take() {
                            let _ = sender.send(found);
                        }
                        if let Some(wv) = app_inner.get_webview(&label_inner) {
                            let _ = wv.close();
                        }
                    });
                },
            ),
            // Keep a normal viewport so layout/lazy-load behave like a real visit; `.hide()`
            // below is what actually keeps it from flashing on screen. The earlier "0 results"
            // runs weren't caused by occlusion/visibility — that was a JSON double-encoding bug
            // in the extraction script (see comment above).
            tauri::PhysicalPosition::new(0, 0),
            tauri::PhysicalSize::new(1280, 900),
        )
        .map_err(|e: tauri::Error| AppError::Settings(e.to_string()))?;
    let _ = outer_webview.hide();

    let outcome = tokio::time::timeout(std::time::Duration::from_secs(20), rx).await;

    // Always clean up the hidden webview, even on timeout/failure.
    if let Some(wv) = app.get_webview(&label) {
        let _ = wv.close();
    }

    match outcome {
        Ok(Ok(images)) => {
            info!("search_images_internal: returning {} results", images.len());
            if images.is_empty() {
                tracing::warn!("search_images_internal: no images found in page DOM");
            }
            Ok(images)
        }
        _ => {
            tracing::warn!("search_images_internal: timed out waiting for image extraction");
            Ok(Vec::new())
        }
    }
}
