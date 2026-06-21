import Header from "./Header.jsx";
import Footer from "./Footer.jsx";

// `doc` is pre-rendered from PRIVACY.md/LICENSE by scripts/build-legal.mjs.
// `doc.text` (plain, e.g. LICENSE) renders in a <pre>; `doc.html` renders as rich text.
export default function LegalPage({ title, doc }) {
  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      <main className="flex-1 max-w-3xl mx-auto px-6 py-16 w-full">
        <h1 className="text-3xl font-bold tracking-tight">{title}</h1>
        {doc.updated && <p className="mt-2 text-sm text-stone-500">Last updated: {doc.updated}</p>}
        <p className="mt-1 text-xs text-stone-600">This document is provided in English only.</p>
        {doc.text ? (
          <pre className="mt-10 text-sm text-stone-300 leading-relaxed whitespace-pre-wrap font-mono">
            {doc.text}
          </pre>
        ) : (
          <div
            className="mt-10 text-[15px] text-stone-300 leading-relaxed space-y-5
              [&_h2]:text-lg [&_h2]:font-semibold [&_h2]:text-stone-100 [&_h2]:mt-10 [&_h2]:mb-3 [&_h2]:first:mt-0
              [&_a]:text-accent-400 [&_a]:hover:underline
              [&_strong]:text-stone-100 [&_strong]:font-semibold
              [&_code]:text-accent-300 [&_code]:bg-stone-900 [&_code]:px-1 [&_code]:py-0.5 [&_code]:rounded [&_code]:text-[13px]
              [&_ul]:list-disc [&_ul]:pl-5 [&_ul]:space-y-2 [&_p]:mb-5"
            dangerouslySetInnerHTML={{ __html: doc.html }}
          />
        )}
      </main>
      <Footer />
    </div>
  );
}
