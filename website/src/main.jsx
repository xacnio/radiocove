import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import { LanguageProvider } from "./lib/LanguageContext.jsx";
import App from "./App.jsx";

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <LanguageProvider routeLang="en">
      <App />
    </LanguageProvider>
  </StrictMode>
);
