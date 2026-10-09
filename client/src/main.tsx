import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <p className="p-8 font-display text-2xl">Button Men</p>
  </StrictMode>,
);
