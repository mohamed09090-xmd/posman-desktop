#!/usr/bin/env python3
"""Browser evidence for the fixture-only POSMAN Phase 03 UI gallery."""

from __future__ import annotations

import json
import os
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path

from playwright.sync_api import Browser, Page, sync_playwright

ROOT = Path(__file__).resolve().parents[2]
ARTIFACT_DIR = Path(os.environ.get("POSMAN_ARTIFACT_DIR", Path(os.environ.get("RUNNER_TEMP", "/tmp")) / "posman-ui-artifacts"))
BASE_URL = "http://127.0.0.1:1420"
AXE_PATH = ROOT / "node_modules" / "axe-core" / "axe.min.js"


def wait_for_server(timeout: float = 45.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", 1420), timeout=0.5):
                return
        except OSError:
            time.sleep(0.25)
    raise RuntimeError("Vite did not start on 127.0.0.1:1420")


def new_page(browser: Browser, width: int, height: int) -> tuple[Page, list[str]]:
    page = browser.new_page(viewport={"width": width, "height": height}, device_scale_factor=1)
    console_errors: list[str] = []
    page.on("console", lambda message: console_errors.append(message.text) if message.type == "error" else None)
    page.on("pageerror", lambda error: console_errors.append(str(error)))
    page.goto(BASE_URL, wait_until="networkidle")
    page.locator("[data-testid='today-screen']").wait_for()
    return page, console_errors


def assert_no_page_overflow(page: Page, label: str) -> None:
    dimensions = page.evaluate(
        """() => ({
          innerWidth: window.innerWidth,
          documentWidth: document.documentElement.scrollWidth,
          bodyWidth: document.body.scrollWidth,
          frameWidth: document.querySelector('.app-frame')?.scrollWidth ?? 0,
        })"""
    )
    widest = max(dimensions["documentWidth"], dimensions["bodyWidth"], dimensions["frameWidth"])
    if widest > dimensions["innerWidth"] + 1:
        raise AssertionError(f"{label}: page-level horizontal overflow {widest}px > {dimensions['innerWidth']}px")


def run_axe(page: Page, name: str) -> list[dict[str, object]]:
    if not AXE_PATH.is_file():
        raise RuntimeError(f"axe-core script is missing: {AXE_PATH}")
    page.add_script_tag(path=str(AXE_PATH))
    result = page.evaluate("""async () => await axe.run(document, { resultTypes: ['violations'] })""")
    violations = result["violations"]
    serious = [item for item in violations if item.get("impact") in {"critical", "serious"}]
    output = ARTIFACT_DIR / "accessibility"
    output.mkdir(parents=True, exist_ok=True)
    (output / f"{name}.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
    if serious:
        summary = ", ".join(f"{item['id']} ({item['impact']})" for item in serious)
        raise AssertionError(f"{name}: axe found critical/serious violations: {summary}")
    return violations


def screenshot(page: Page, filename: str) -> None:
    target = ARTIFACT_DIR / "screenshots" / filename
    target.parent.mkdir(parents=True, exist_ok=True)
    page.screenshot(path=str(target), full_page=False)


def validate_keyboard_path(page: Page) -> None:
    page.keyboard.press("Tab")
    focused = page.evaluate("document.activeElement?.className")
    if "skip-link" not in str(focused):
        raise AssertionError(f"First focus target is not the skip link: {focused!r}")
    page.keyboard.press("Enter")
    active_id = page.evaluate("document.activeElement?.id")
    if active_id != "main-content":
        raise AssertionError(f"Skip link did not focus main content: {active_id!r}")
    page.keyboard.press("Tab")
    if page.evaluate("document.activeElement === document.body"):
        raise AssertionError("Keyboard focus returned unexpectedly to the body")


def main() -> int:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    vite_log = ARTIFACT_DIR / "vite.log"
    log_handle = vite_log.open("w", encoding="utf-8")
    process = subprocess.Popen(
        ["npm", "run", "dev", "--", "--host", "127.0.0.1", "--port", "1420"],
        cwd=ROOT,
        stdout=log_handle,
        stderr=subprocess.STDOUT,
        text=True,
        start_new_session=True,
    )
    try:
        wait_for_server()
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch()
            try:
                page, errors = new_page(browser, 1280, 800)
                assert page.locator("html").get_attribute("lang") == "ar-DZ"
                assert page.locator("html").get_attribute("dir") == "rtl"
                validate_keyboard_path(page)
                assert_no_page_overflow(page, "Arabic Today 1280x800")
                run_axe(page, "arabic-today")
                screenshot(page, "01-arabic-today-1280x800.png")

                page.locator("[data-testid='language-switch']").click()
                assert page.locator("html").get_attribute("lang") == "fr-DZ"
                assert page.locator("html").get_attribute("dir") == "ltr"
                page.locator("text=Registre des opérations du jour").wait_for()
                assert_no_page_overflow(page, "French Today 1280x800")
                run_axe(page, "french-today")
                screenshot(page, "02-french-today-1280x800.png")
                if errors:
                    raise AssertionError(f"Console errors on Today screens: {errors}")
                page.close()

                page, errors = new_page(browser, 1024, 640)
                page.locator("[data-workspace='sales']").click()
                page.locator("[data-view='invoice']").wait_for()
                page.locator("[data-testid='invoice-grid']").wait_for()
                assert_no_page_overflow(page, "Arabic Invoice 1024x640")
                run_axe(page, "arabic-invoice")
                screenshot(page, "03-arabic-invoice-1024x640.png")
                if errors:
                    raise AssertionError(f"Console errors on Invoice screen: {errors}")
                page.close()

                page, errors = new_page(browser, 1440, 900)
                page.locator("[data-workspace='inventory']").click()
                page.locator("[data-testid='product-grid'] tbody tr").first.click()
                page.locator("[data-testid='product-drawer']").wait_for()
                assert_no_page_overflow(page, "Product list and drawer 1440x900")
                run_axe(page, "product-list-drawer")
                screenshot(page, "04-product-list-drawer-1440x900.png")
                page.locator("[data-testid='product-search']").fill("not-a-product")
                page.locator("text=لا توجد مواد مطابقة").wait_for()
                if errors:
                    raise AssertionError(f"Console errors on Product screen: {errors}")
                page.close()

                page, errors = new_page(browser, 1280, 800)
                page.locator("[data-workspace='sales']").click()
                page.locator("[data-view='sales-cycle']").click()
                page.locator(".process-strip").wait_for()
                assert page.locator(".process-step").count() == 4
                assert_no_page_overflow(page, "Sales Cycle 1280x800")
                run_axe(page, "sales-cycle")
                screenshot(page, "05-sales-cycle-1280x800.png")
                if errors:
                    raise AssertionError(f"Console errors on Sales Cycle screen: {errors}")
                page.close()
            finally:
                browser.close()

        print(f"UI browser evidence passed. Artifacts: {ARTIFACT_DIR}")
        return 0
    except Exception as error:
        log_handle.flush()
        print(f"UI browser evidence failed: {error}", file=sys.stderr)
        if vite_log.is_file():
            output = vite_log.read_text(encoding="utf-8", errors="replace")
            if output:
                print("--- Vite output ---", file=sys.stderr)
                print(output, file=sys.stderr)
        return 1
    finally:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        log_handle.close()


if __name__ == "__main__":
    raise SystemExit(main())
