import { useEffect } from "react";
import releaseData from "./data/releases.json";
import Header from "./components/Header.jsx";
import Hero from "./components/Hero.jsx";
import Screenshots from "./components/Screenshots.jsx";
import Features from "./components/Features.jsx";
import MediaControls from "./components/MediaControls.jsx";
import Download from "./components/Download.jsx";
import Changelog from "./components/Changelog.jsx";
import Footer from "./components/Footer.jsx";

export default function App() {
  const releases = releaseData.releases ?? [];
  const latestRelease = releases.find((r) => !r.prerelease) ?? releases[0] ?? null;
  const latestVersion = latestRelease?.tag?.replace(/^v/, "");

  // Re-scroll to the hash once mounted: native scroll-on-load can fire before this section renders.
  useEffect(() => {
    if (!window.location.hash) return;
    const id = window.location.hash.slice(1);
    document.getElementById(id)?.scrollIntoView();
  }, []);

  return (
    <div className="min-h-screen">
      <Header />
      <main>
        <Hero latestVersion={latestVersion} downloads={latestRelease?.downloads ?? []} />
        <Screenshots />
        <Features />
        <MediaControls />
        <Download latestRelease={latestRelease} />
        <Changelog releases={releases} />
      </main>
      <Footer />
    </div>
  );
}
