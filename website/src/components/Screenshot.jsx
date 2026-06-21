import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import ScreenshotPlaceholder from "./ScreenshotPlaceholder.jsx";

function Lightbox({ src, alt, onClose }) {
  useEffect(() => {
    const onKey = (e) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return createPortal(
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-stone-950/90 backdrop-blur-sm p-6 cursor-zoom-out"
      onClick={onClose}
    >
      <img
        src={src}
        alt={alt}
        className="max-w-full max-h-full rounded-xl border border-stone-700/60 shadow-2xl shadow-black/50 cursor-default"
        onClick={(e) => e.stopPropagation()}
      />
    </div>,
    document.body
  );
}

// Falls back to a labeled placeholder if the screenshot file is missing.
export default function Screenshot({ src, alt, placeholder, note, className = "" }) {
  const [failed, setFailed] = useState(false);
  const [open, setOpen] = useState(false);
  const fullSrc = `${import.meta.env.BASE_URL}${src}`;

  if (failed) {
    return <ScreenshotPlaceholder label={placeholder} note={note} className={className} />;
  }

  return (
    <>
      <img
        src={fullSrc}
        alt={alt}
        onError={() => setFailed(true)}
        onClick={() => setOpen(true)}
        className={`w-full rounded-xl border border-stone-700/60 shadow-2xl shadow-black/50 cursor-zoom-in transition-[border-color] hover:border-accent-500/60 ${className}`}
      />
      {open && <Lightbox src={fullSrc} alt={alt} onClose={() => setOpen(false)} />}
    </>
  );
}
