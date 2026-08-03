import { I18nProvider } from "../i18n/I18nProvider";
import { Phase05App } from "../features/phase05/Phase05App";
import "../styles/tokens.css";
import "../styles/ui-foundation.css";

export function AppRoot() {
  return <I18nProvider><Phase05App /></I18nProvider>;
}
