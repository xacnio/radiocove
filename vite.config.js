import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { copyFileSync, mkdirSync, readdirSync, readFileSync, existsSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

const host = process.env.TAURI_DEV_HOST;

/** Serve src/locales/*.json at /locales/ so the splash screen can fetch them */
function localesPlugin() {
    return {
        name: 'serve-locales',
        configureServer(server) {
            server.middlewares.use('/locales', (req, res, next) => {
                try {
                    const file = resolve('src/locales', req.url.slice(1));
                    if (existsSync(file)) {
                        res.setHeader('Content-Type', 'application/json');
                        res.end(readFileSync(file, 'utf-8'));
                        return;
                    }
                } catch { /* fall through */ }
                next();
            });
        },
        writeBundle(options) {
            const src = resolve('src/locales');
            const dest = join(options.dir || 'dist', 'locales');
            mkdirSync(dest, { recursive: true });
            for (const f of readdirSync(src).filter(n => n.endsWith('.json'))) {
                copyFileSync(join(src, f), join(dest, f));
            }
        }
    };
}

export default defineConfig({
    plugins: [react(), localesPlugin()],
    clearScreen: false,
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
        watch: { ignored: ['**/src-tauri/**'] },
    },
    build: {
        chunkSizeWarningLimit: 1000,
        rollupOptions: {
            // Tray window gets its own minimal entry (no i18n/station-list/full app bundle)
            // so it's fast to (re)load after the idle-destroy poller tears it down.
            input: {
                main: resolve(__dirname, 'index.html'),
                tray: resolve(__dirname, 'tray.html'),
            },
            output: {
                manualChunks(id) {
                    if (id.includes('node_modules')) {
                        if (id.includes('react') || id.includes('react-dom')) {
                            return 'vendor';
                        }
                        if (id.includes('@tauri-apps/api')) {
                            return 'tauri';
                        }
                        if (id.includes('lucide-react')) {
                            return 'icons';
                        }
                    }
                }
            }
        }
    }
});
