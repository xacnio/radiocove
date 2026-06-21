import { FiImage } from "react-icons/fi";

// Visually distinct placeholder for a screenshot that still needs capturing.
export default function ScreenshotPlaceholder({ label, note, className = "" }) {
  return (
    <div
      className={`min-h-[220px] rounded-xl border border-dashed border-stone-700 bg-stone-900/40 flex flex-col items-center justify-center gap-2.5 text-center px-8 ${className}`}
    >
      <FiImage size={22} className="text-stone-600" />
      {note && <p className="text-[10px] font-medium uppercase tracking-wide text-stone-600">{note}</p>}
      <p className="text-xs text-stone-500 max-w-[16rem] leading-relaxed">{label}</p>
    </div>
  );
}
