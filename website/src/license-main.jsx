import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import { LanguageProvider } from "./lib/LanguageContext.jsx";
import LicensePage from "./pages/LicensePage.jsx";

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <LanguageProvider>
      <LicensePage />
    </LanguageProvider>
  </StrictMode>
);
