import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import { LanguageProvider } from "./lib/LanguageContext.jsx";
import PrivacyPage from "./pages/PrivacyPage.jsx";

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <LanguageProvider>
      <PrivacyPage />
    </LanguageProvider>
  </StrictMode>
);
