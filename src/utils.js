import { convertFileSrc } from '@tauri-apps/api/core';

/**
 * Convert a favicon URL for display in the webview.
 * file:/// URLs are converted to Tauri asset protocol URLs.
 * Remote URLs are passed through unchanged.
 */
export function toAssetUrl(url) {
    if (!url) return '';
    if (url.startsWith('file:///')) {
        const path = url.slice(8); // strip "file:///"
        return convertFileSrc(path);
    }
    return url;
}

/** Compares two "x.y.z" version strings (optional "v" prefix). Returns -1, 0 or 1. */
export function compareVersions(a, b) {
    const pa = a.replace(/^v/, '').split('.').map(Number);
    const pb = b.replace(/^v/, '').split('.').map(Number);
    for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
        const na = pa[i] || 0, nb = pb[i] || 0;
        if (na !== nb) return na - nb;
    }
    return 0;
}

/** Pulls just the "### 📝 What's New" section out of a CI-generated release body. */
export function extractWhatsNew(body) {
    if (!body) return '';
    const match = body.match(/### 📝 What's New\s+([\s\S]*?)(?=\n---|###|$)/);
    return match ? match[1].trim() : '';
}
