import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { AppRoot } from "./app/AppRoot";
import "./bootstrap/bootstrap.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("POSMAN bootstrap root element was not found.");
}

createRoot(rootElement).render(
  <StrictMode>
    <AppRoot />
  </StrictMode>,
);
