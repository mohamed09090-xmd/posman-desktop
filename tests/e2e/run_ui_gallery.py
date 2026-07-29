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
from typing import TypedDict

from playwright.sync_api import Browser, Page, sync_playwright

ROOT = Path(__file__).resolve().parents[2]
ARTIFACT_DIR = Path(os.environ.get("POSMAN_ARTIFACT_DIR", Path(os.environ.get("RUNNER_TEMP", "/tmp")) / "posman-ui-artifacts"))
BASE_URL = "http://127.0.0.1:1420"
AXE_PATH = ROOT / "node_modules" / "axe-core" / "axe.min.js"


class AxeSummary(TypedDict):
    violations: int
    incomplete: int
    unresolved_critical_serious_incomplete: int
    passes: int


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


def assert_command_bar_visible(page: Page, label: str) -> None:
    result = page.locator(".command-bar").evaluate(
        """(element) => {
          const rect = element.getBoundingClientRect();
          return {
            top: rect.top,
            left: rect.left,
            right: rect.right,
            bottom: rect.bottom,
            width: rect.width,
            height: rect.height,
            viewportWidth: window.innerWidth,
            viewportHeight: window.innerHeight,
          };
        }"""
    )
    if (
        result["width"] <= 0
        or result["height"] <= 0
        or result["top"] < -1
        or result["left"] < -1
        or result["right"] > result["viewportWidth"] + 1
        or result["bottom"] > result["viewportHeight"] + 1
    ):
        raise AssertionError(f"{label}: CommandBar is not fully visible: {result}")


def assert_workspace_labels_visible(page: Page, label: str) -> None:
    results = page.locator(".workspace-rail__label").evaluate_all(
        """(elements) => elements.map((element) => {
          const style = getComputedStyle(element);
          const rect = element.getBoundingClientRect();
          return {
            text: element.textContent?.trim() ?? '',
            clientWidth: element.clientWidth,
            scrollWidth: element.scrollWidth,
            clientHeight: element.clientHeight,
            scrollHeight: element.scrollHeight,
            rectWidth: rect.width,
            rectHeight: rect.height,
            overflow: style.overflow,
            textOverflow: style.textOverflow,
            whiteSpace: style.whiteSpace,
          };
        })"""
    )
    if len(results) != 7:
        raise AssertionError(f"{label}: expected 7 workspace labels, found {len(results)}")

    failures = []
    for result in results:
        clipped = (
            not result["text"]
            or result["rectWidth"] <= 0
            or result["rectHeight"] <= 0
            or result["scrollWidth"] > result["clientWidth"] + 1
            or result["scrollHeight"] > result["clientHeight"] + 1
            or result["textOverflow"] == "ellipsis"
            or result["whiteSpace"] == "nowrap"
        )
        if clipped:
            failures.append(result)
    if failures:
        raise AssertionError(f"{label}: workspace labels are visually clipped: {failures}")


def run_axe(page: Page, name: str) -> AxeSummary:
    if not AXE_PATH.is_file():
        raise RuntimeError(f"axe-core script is missing: {AXE_PATH}")
    page.add_script_tag(path=str(AXE_PATH))
    result = page.evaluate(
        """async () => await axe.run(document, {
          resultTypes: ['violations', 'incomplete', 'passes'],
        })"""
    )
    violations = result.get("violations", [])
    incomplete = result.get("incomplete", [])
    unresolved = [item for item in incomplete if item.get("impact") in {"critical", "serious"}]
    output = ARTIFACT_DIR / "accessibility"
    output.mkdir(parents=True, exist_ok=True)
    (output / f"{name}.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")

    summary: AxeSummary = {
        "violations": len(violations),
        "incomplete": len(incomplete),
        "unresolved_critical_serious_incomplete": len(unresolved),
        "passes": len(result.get("passes", [])),
    }
    if violations or unresolved:
        violation_summary = ", ".join(f"{item['id']} ({item.get('impact')})" for item in violations) or "none"
        incomplete_summary = ", ".join(f"{item['id']} ({item.get('impact')})" for item in unresolved) or "none"
        raise AssertionError(
            f"{name}: axe violations={violation_summary}; unresolved critical/serious incomplete={incomplete_summary}"
        )
    return summary


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
    axe_summaries: dict[str, AxeSummary] = {}
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
                keyboard_page, keyboard_errors = new_page(browser, 1280, 800)
                validate_keyboard_path(keyboard_page)
                if keyboard_errors:
                    raise AssertionError(f"Console errors during Skip Link validation: {keyboard_errors}")
                keyboard_page.close()

                page, errors = new_page(browser, 1280, 800)
                assert page.locator("html").get_attribute("lang") == "ar-DZ"
                assert page.locator("html").get_attribute("dir") == "rtl"
                assert_command_bar_visible(page, "Arabic Today 1280x800")
                assert_workspace_labels_visible(page, "Arabic rail 1280x800")
                assert_no_page_overflow(page, "Arabic Today 1280x800")
                axe_summaries["arabic-today"] = run_axe(page, "arabic-today")
                screenshot(page, "01-arabic-today-1280x800.png")

                page.locator("[data-testid='language-switch']").click()
                assert page.locator("html").get_attribute("lang") == "fr-DZ"
                assert page.locator("html").get_attribute("dir") == "ltr"
                page.locator("text=Registre des opérations du jour").wait_for()
                assert_workspace_labels_visible(page, "French rail 1280x800")
                assert_no_page_overflow(page, "French Today 1280x800")
                axe_summaries["french-today"] = run_axe(page, "french-today")
                screenshot(page, "02-french-today-1280x800.png")
                if errors:
                    raise AssertionError(f"Console errors on Today screens: {errors}")
                page.close()

                page, errors = new_page(browser, 1024, 640)
                assert_workspace_labels_visible(page, "Arabic rail 1024x640")
                page.locator("[data-testid='language-switch']").click()
                page.locator("text=Registre des opérations du jour").wait_for()
                assert_workspace_labels_visible(page, "French rail 1024x640")
                page.locator("[data-testid='language-switch']").click()
                page.locator("[data-testid='today-screen']").wait_for()
                assert page.locator("html").get_attribute("lang") == "ar-DZ"
                page.locator("[data-workspace='sales']").click()
                page.locator("[data-view='invoice']").wait_for()
                page.locator("[data-testid='invoice-grid']").wait_for()
                assert_no_page_overflow(page, "Arabic Invoice 1024x640")
                axe_summaries["arabic-invoice"] = run_axe(page, "arabic-invoice")
                screenshot(page, "03-arabic-invoice-1024x640.png")
                if errors:
                    raise AssertionError(f"Console errors on Invoice screen: {errors}")
                page.close()

                page, errors = new_page(browser, 1440, 900)
                page.locator("[data-workspace='inventory']").click()
                page.locator("[data-testid='product-grid'] tbody tr").first.click()
                page.locator("[data-testid='product-drawer']").wait_for()
                assert_no_page_overflow(page, "Product list and drawer 1440x900")
                axe_summaries["product-list-drawer"] = run_axe(page, "product-list-drawer")
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
                axe_summaries["sales-cycle"] = run_axe(page, "sales-cycle")
                screenshot(page, "05-sales-cycle-1280x800.png")
                if errors:
                    raise AssertionError(f"Console errors on Sales Cycle screen: {errors}")
                page.close()
            finally:
                browser.close()

        accessibility_dir = ARTIFACT_DIR / "accessibility"
        accessibility_dir.mkdir(parents=True, exist_ok=True)
        (accessibility_dir / "summary.json").write_text(
            json.dumps(axe_summaries, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
        print(json.dumps(axe_summaries, ensure_ascii=False, sort_keys=True))
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
