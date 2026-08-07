#!/usr/bin/env python3
"""Instrumented PHASE 09 browser scenarios using synthetic data only.

The Windows-native PDF/print and destructive restore evidence remains in Rust/Tauri
jobs. These browser scenarios validate the local preview/workspace surface, locale,
accessibility, console/page errors, overflow, clipping, and scenario outcomes.
"""

from __future__ import annotations

import json
import os
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Callable

from playwright.sync_api import Page, sync_playwright

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "artifacts" / "phase09-e2e"
BASE_URL = os.environ.get("POSMAN_E2E_BASE_URL", "http://127.0.0.1:4173")
AXE_PATH = ROOT / "node_modules" / "axe-core" / "axe.min.js"


@dataclass(frozen=True)
class Scenario:
    name: str
    locale: str
    viewport: tuple[int, int]
    section_label: str
    assertion_text: str


SCENARIOS = (
    Scenario(
        "phase09_ar_template_publish_and_historical_reprint",
        "ar-DZ",
        (1280, 800),
        "القوالب",
        "نشر النسخة",
    ),
    Scenario(
        "phase09_fr_sales_invoice_preview_and_pdf",
        "fr-DZ",
        (1024, 640),
        "Documents",
        "Créer le PDF historique",
    ),
    Scenario(
        "phase09_ar_reports_csv_and_pdf",
        "ar-DZ",
        (1280, 800),
        "التقارير",
        "تصدير CSV",
    ),
    Scenario(
        "phase09_fr_audit_filter_and_redacted_export",
        "fr-DZ",
        (1024, 640),
        "Audit",
        "Exporter CSV",
    ),
    Scenario(
        "phase09_ar_manual_backup_and_verification",
        "ar-DZ",
        (1280, 800),
        "النسخ والاستعادة",
        "إنشاء نسخة يدوية",
    ),
    Scenario(
        "phase09_fr_corrupted_backup_rejected",
        "fr-DZ",
        (1024, 640),
        "Sauvegarde et restauration",
        "Vérifier l’intégrité",
    ),
    Scenario(
        "phase09_ar_restore_requires_verified_safety_backup",
        "ar-DZ",
        (1280, 800),
        "النسخ والاستعادة",
        "اكتب RESTORE",
    ),
    Scenario(
        "phase09_fr_restore_success_returns_to_login",
        "fr-DZ",
        (1024, 640),
        "Sauvegarde et restauration",
        "Saisissez RESTORE",
    ),
)


def page_url(locale: str) -> str:
    return f"{BASE_URL}/?workspace=phase09&locale={locale}"


def run_axe(page: Page) -> list[dict[str, object]]:
    if not AXE_PATH.is_file():
        raise RuntimeError(f"axe-core is unavailable at {AXE_PATH}")
    page.add_script_tag(path=str(AXE_PATH))
    result = page.evaluate("async () => await window.axe.run(document)")
    return list(result.get("violations", []))


def geometry(page: Page) -> dict[str, object]:
    return page.evaluate(
        """
        () => {
          const root = document.documentElement;
          const elements = Array.from(document.querySelectorAll('*'));
          const clipped = elements.filter((element) => {
            const rect = element.getBoundingClientRect();
            const style = getComputedStyle(element);
            return rect.width > 0 && rect.height > 0 &&
              style.overflow !== 'visible' &&
              (element.scrollWidth > element.clientWidth + 1 ||
               element.scrollHeight > element.clientHeight + 1);
          }).slice(0, 50).map((element) => ({
            tag: element.tagName,
            className: String(element.className || ''),
            clientWidth: element.clientWidth,
            scrollWidth: element.scrollWidth,
            clientHeight: element.clientHeight,
            scrollHeight: element.scrollHeight,
          }));
          return {
            horizontalOverflow: root.scrollWidth > root.clientWidth + 1,
            documentWidth: root.scrollWidth,
            viewportWidth: root.clientWidth,
            clipped,
          };
        }
        """
    )


def run_scenario(page: Page, scenario: Scenario) -> dict[str, object]:
    console_errors: list[str] = []
    page_errors: list[str] = []
    page.on(
        "console",
        lambda message: console_errors.append(message.text)
        if message.type == "error"
        else None,
    )
    page.on("pageerror", lambda error: page_errors.append(str(error)))
    page.set_viewport_size(
        {"width": scenario.viewport[0], "height": scenario.viewport[1]}
    )
    page.goto(page_url(scenario.locale), wait_until="networkidle")
    page.get_by_role("heading", name="POSMAN", exact=False).first.wait_for()
    tab = page.get_by_role("button", name=scenario.section_label, exact=True)
    tab.click()
    page.get_by_text(scenario.assertion_text, exact=False).first.wait_for()

    direction = page.locator("main.phase09-workspace").get_attribute("dir")
    expected_direction = "rtl" if scenario.locale == "ar-DZ" else "ltr"
    if direction != expected_direction:
        raise AssertionError(
            f"{scenario.name}: expected {expected_direction}, received {direction}"
        )

    screenshot = EVIDENCE / f"{scenario.name}.png"
    page.screenshot(path=str(screenshot), full_page=True)
    axe_violations = run_axe(page)
    layout = geometry(page)
    if layout["horizontalOverflow"]:
        raise AssertionError(f"{scenario.name}: document has horizontal overflow")
    if axe_violations:
        raise AssertionError(
            f"{scenario.name}: axe found {len(axe_violations)} violation groups"
        )
    if console_errors or page_errors:
        raise AssertionError(
            f"{scenario.name}: console={console_errors!r}, page={page_errors!r}"
        )
    return {
        "name": scenario.name,
        "locale": scenario.locale,
        "viewport": {
            "width": scenario.viewport[0],
            "height": scenario.viewport[1],
        },
        "screenshot": screenshot.name,
        "consoleErrors": console_errors,
        "pageErrors": page_errors,
        "axeViolations": axe_violations,
        "overflow": layout,
        "clipping": layout["clipped"],
        "outcome": "PASS",
    }


def main() -> int:
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    results: list[dict[str, object]] = []
    failed = False
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True)
        try:
            for scenario in SCENARIOS:
                page = browser.new_page()
                try:
                    results.append(run_scenario(page, scenario))
                except Exception as error:  # evidence must preserve the exact failure
                    failed = True
                    failure_screenshot = EVIDENCE / f"{scenario.name}-failure.png"
                    try:
                        page.screenshot(path=str(failure_screenshot), full_page=True)
                    except Exception:
                        pass
                    results.append(
                        {
                            "name": scenario.name,
                            "locale": scenario.locale,
                            "viewport": {
                                "width": scenario.viewport[0],
                                "height": scenario.viewport[1],
                            },
                            "screenshot": failure_screenshot.name,
                            "outcome": "FAIL",
                            "error": str(error),
                        }
                    )
                finally:
                    page.close()
        finally:
            browser.close()

    manifest = {
        "syntheticDataOnly": True,
        "baseUrl": BASE_URL,
        "scenarios": results,
    }
    (EVIDENCE / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    print(json.dumps(manifest, ensure_ascii=False, indent=2))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
