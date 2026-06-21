import { Sparkles, X } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import ReactMarkdown from 'react-markdown';
import { extractWhatsNew } from '../../utils';

const linkRenderer = {
    a: ({ href, children }) => (
        <a
            href="#"
            onClick={(e) => { e.preventDefault(); if (href) invoke('open_browser_url', { url: href }); }}
            className="text-accent hover:underline cursor-pointer"
        >
            {children}
        </a>
    )
};

export default function WhatsNewModal({ releases, currentVersion, onClose }) {
    const { t } = useTranslation();

    if (!releases || releases.length === 0) return null;

    return (
        <div className="fixed inset-0 bg-black/80 z-[10000] flex items-center justify-center p-4 backdrop-blur-md animate-in fade-in duration-300">
            <div
                className="ctx-menu-container bg-bg-secondary border border-accent/20 shadow-accent/5 rounded-2xl w-full max-w-lg flex flex-col shadow-[0_20px_50px_rgba(0,0,0,0.5)] overflow-hidden animate-in zoom-in duration-300"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="px-6 py-5 flex items-center gap-4 bg-accent/5 border-b border-border/50">
                    <div className="w-12 h-12 rounded-2xl flex items-center justify-center shrink-0 bg-accent/20 text-accent shadow-[0_0_20px_rgba(var(--accent),0.2)]">
                        <Sparkles size={24} />
                    </div>
                    <div className="flex-1 min-w-0">
                        <h2 className="text-base font-bold text-text-primary truncate">{t('whatsNew.title')}</h2>
                        <p className="text-xs text-text-muted">{t('whatsNew.subtitle', { version: currentVersion })}</p>
                    </div>
                    <button onClick={onClose} className="p-2 -mr-2 text-text-muted hover:text-text-primary hover:bg-bg-surface-hover rounded-full transition-all">
                        <X size={20} />
                    </button>
                </div>

                <div className="p-6 max-h-[60vh] overflow-y-auto scrollbar-thin scrollbar-thumb-border/50 space-y-8">
                    {releases.map((release) => (
                        <div key={release.id ?? release.tag_name} className="space-y-3">
                            <div className="flex items-center gap-3">
                                <h5 className="font-bold text-sm text-text-primary">{release.name || release.tag_name}</h5>
                                {release.published_at && (
                                    <span className="text-[10px] uppercase font-black tracking-wider text-accent bg-accent/10 px-2 py-0.5 rounded-lg shrink-0">
                                        {new Date(release.published_at).toLocaleDateString()}
                                    </span>
                                )}
                            </div>
                            <div className="text-[13px] text-text-muted prose prose-sm prose-invert max-w-none prose-headings:text-text-primary prose-a:text-accent prose-strong:text-text-primary prose-p:text-text-muted prose-li:text-text-muted break-words leading-relaxed pl-3 border-l-2 border-border/30">
                                <ReactMarkdown components={linkRenderer}>
                                    {extractWhatsNew(release.body) || release.body || ''}
                                </ReactMarkdown>
                            </div>
                        </div>
                    ))}
                </div>

                <div className="p-6 pt-0">
                    <button
                        onClick={onClose}
                        className="w-full py-3 rounded-xl font-bold text-xs transition-all text-white shadow-xl"
                        style={{
                            background: 'linear-gradient(to right, rgb(var(--accent)), rgb(var(--accent) / 0.8))',
                            boxShadow: '0 10px 15px -3px rgb(var(--accent) / 0.3)'
                        }}
                    >
                        {t('whatsNew.gotIt')}
                    </button>
                </div>
            </div>
        </div>
    );
}
