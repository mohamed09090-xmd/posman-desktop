#!/usr/bin/env python3
"""Browser, accessibility, and runtime-integration evidence for POSMAN."""

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
ARTIFACT_DIR = Path(
    os.environ.get(
        "POSMAN_ARTIFACT_DIR",
        Path(os.environ.get("RUNNER_TEMP", "/tmp")) / "posman-ui-artifacts",
    )
)
BASE_URL = "http://127.0.0.1:1420"
AXE_PATH = ROOT / "node_modules" / "axe-core" / "axe.min.js"

READY_PAYLOAD = {
    "databaseReady": True,
    "schemaVersion": "0005",
    "migrationCount": 5,
    "foreignKeysEnabled": True,
    "journalMode": "wal",
}


class AxeSummary(TypedDict):
    violations: int
    incomplete: int
    unresolved_critical_serious_incomplete: int
    passes: int


def runtime_script(mode: str) -> str:
    payload = json.dumps(READY_PAYLOAD)
    if mode == "ready":
        return f"""
        window.__POSMAN_RUNTIME_CALLS__ = [];
        window.__POSMAN_DEV_RUNTIME_INVOKER__ = async function(command, args) {{
          window.__POSMAN_RUNTIME_CALLS__.push({{ command, args, argumentCount: arguments.length }});
          return {payload};
        }};
        """
    if mode == "error-retry":
        return f"""
        window.__POSMAN_RUNTIME_CALLS__ = [];
        window.__POSMAN_DEV_RUNTIME_INVOKER__ = async function(command, args) {{
          window.__POSMAN_RUNTIME_CALLS__.push({{ command, args, argumentCount: arguments.length }});
          if (window.__POSMAN_RUNTIME_CALLS__.length === 1) {{
            throw {{
              code: "RUNTIME_STATUS_UNAVAILABLE",
              message: "C:\\\\Users\\\\merchant\\\\POSMAN\\\\data\\\\posman.sqlite3 SELECT failed"
            }};
          }}
          await new Promise((resolve) => setTimeout(resolve, 150));
          return {payload};
        }};
        """
    if mode == "malformed":
        return """
        window.__POSMAN_RUNTIME_CALLS__ = [];
        window.__POSMAN_DEV_RUNTIME_INVOKER__ = async function(command, args) {
          window.__POSMAN_RUNTIME_CALLS__.push({ command, args, argumentCount: arguments.length });
          return {
            databaseReady: true,
            schemaVersion: "0005",
            migrationCount: "5",
            foreignKeysEnabled: true,
            journalMode: "wal"
          };
        };
        """
    raise ValueError(f"Unknown runtime mode: {mode}")


def wait_for_server(timeout: float = 45.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", 1420), timeout=0.5):
                return
        except OSError:
            time.sleep(0.25)
    raise RuntimeError("Vite did not start on 127.0.0.1:1420")


def new_page(
    browser: Browser,
    width: int,
    height: int,
    init_script: str | None = None,
) -> tuple[Page, list[str]]:
    page = browser.new_page(viewport={"width": width, "height": height}, device_scale_factor=1)
    console_errors: list[str] = []
    page.on(
        "console",
        lambda message: console_errors.append(message.text)
        if message.type == "error"
        else None,
    )
    page.on("pageerror", lambda error: console_errors.append(str(error)))
    if init_script:
        page.add_init_script(init_script)
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
        raise AssertionError(
            f"{label}: page-level horizontal overflow {widest}px > {dimensions['innerWidth']}px"
        )


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


def assert_runtime_status_visible(page: Page, label: str) -> None:
    result = page.locator(".runtime-status").evaluate(
        """(element) => {
          const rect = element.getBoundingClientRect();
          const host = element.closest('.command-bar');
          const hostRect = host?.getBoundingClientRect();
          return {
            text: element.textContent?.trim() ?? '',
            width: rect.width,
            height: rect.height,
            left: rect.left,
            right: rect.right,
            top: rect.top,
            bottom: rect.bottom,
            hostLeft: hostRect?.left ?? 0,
            hostRight: hostRect?.right ?? 0,
            hostTop: hostRect?.top ?? 0,
            hostBottom: hostRect?.bottom ?? 0,
          };
        }"""
    )
    if (
        not result["text"]
        or result["width"] <= 0
        or result["height"] <= 0
        or result["left"] < result["hostLeft"] - 1
        or result["right"] > result["hostRight"] + 1
        or result["top"] < result["hostTop"] - 1
        or result["bottom"] > result["hostBottom"] + 1
    ):
        raise AssertionError(f"{label}: runtime status is clipped or invisible: {result}")


def assert_runtime_primary_unclipped(page: Page, label: str) -> None:
    result = page.locator("[data-testid='runtime-status-primary']").evaluate(
        """(element) => {
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
            textOverflow: style.textOverflow,
            whiteSpace: style.whiteSpace,
          };
        }"""
    )
    if (
        not result["text"]
        or result["rectWidth"] <= 0
        or result["rectHeight"] <= 0
        or result["scrollWidth"] > result["clientWidth"] + 1
        or result["scrollHeight"] > result["clientHeight"] + 1
        or result["textOverflow"] == "ellipsis"
        or result["whiteSpace"] == "nowrap"
    ):
        raise AssertionError(f"{label}: primary runtime status is clipped: {result}")


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


def assert_runtime_call_contract(page: Page, expected_calls: int) -> None:
    calls = page.evaluate("window.__POSMAN_RUNTIME_CALLS__")
    if len(calls) != expected_calls:
        raise AssertionError(f"Expected {expected_calls} runtime calls, found {calls}")
    for call in calls:
        if call["command"] != "get_runtime_status" or call["argumentCount"] != 1:
            raise AssertionError(f"Unexpected runtime invocation: {call}")
        if call.get("args") is not None:
            raise AssertionError(f"Runtime invocation included unexpected arguments: {call}")


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
    (output / f"{name}.json").write_text(
        json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8"
    )

    summary: AxeSummary = {
        "violations": len(violations),
        "incomplete": len(incomplete),
        "unresolved_critical_serious_incomplete": len(unresolved),
        "passes": len(result.get("passes", [])),
    }
    if violations or unresolved:
        violation_summary = ", ".join(
            f"{item['id']} ({item.get('impact')})" for item in violations
        ) or "none"
        incomplete_summary = ", ".join(
            f"{item['id']} ({item.get('impact')})" for item in unresolved
        ) or "none"
        raise AssertionError(
            f"{name}: axe violations={violation_summary}; "
            f"unresolved critical/serious incomplete={incomplete_summary}"
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


def assert_clean_console(errors: list[str], label: str) -> None:
    if errors:
        raise AssertionError(f"Console errors on {label}: {errors}")


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
                assert_clean_console(keyboard_errors, "Skip Link validation")
                keyboard_page.close()

                page, errors = new_page(browser, 1280, 800)
                assert page.locator("html").get_attribute("lang") == "ar-DZ"
                assert page.locator("html").get_attribute("dir") == "rtl"
                assert_command_bar_visible(page, "Arabic Today 1280x800")
                assert_runtime_status_visible(page, "Arabic preview 1280x800")
                assert_runtime_primary_unclipped(page, "Arabic preview 1280x800")
                assert_workspace_labels_visible(page, "Arabic rail 1280x800")
                assert_no_page_overflow(page, "Arabic Today 1280x800")
                axe_summaries["arabic-today"] = run_axe(page, "arabic-today")
                screenshot(page, "01-arabic-today-1280x800.png")

                page.locator("[data-testid='language-switch']").click()
                assert page.locator("html").get_attribute("lang") == "fr-DZ"
                assert page.locator("html").get_attribute("dir") == "ltr"
                page.locator("text=Registre des opérations du jour").wait_for()
                assert_runtime_primary_unclipped(page, "French preview 1280x800")
                assert_workspace_labels_visible(page, "French rail 1280x800")
                assert_no_page_overflow(page, "French Today 1280x800")
                axe_summaries["french-today"] = run_axe(page, "french-today")
                screenshot(page, "02-french-today-1280x800.png")
                assert_clean_console(errors, "Today screens")
                page.close()

                page, errors = new_page(browser, 1024, 640)
                assert_runtime_primary_unclipped(page, "Arabic preview 1024x640")
                assert_workspace_labels_visible(page, "Arabic rail 1024x640")
                page.locator("[data-testid='language-switch']").click()
                page.locator("text=Registre des opérations du jour").wait_for()
                assert_runtime_primary_unclipped(page, "French preview 1024x640")
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
                assert_clean_console(errors, "Invoice screen")
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
                assert_clean_console(errors, "Product screen")
                page.close()

                page, errors = new_page(browser, 1280, 800)
                page.locator("[data-workspace='sales']").click()
                page.locator("[data-view='sales-cycle']").click()
                page.locator(".process-strip").wait_for()
                assert page.locator(".process-step").count() == 4
                assert_no_page_overflow(page, "Sales Cycle 1280x800")
                axe_summaries["sales-cycle"] = run_axe(page, "sales-cycle")
                screenshot(page, "05-sales-cycle-1280x800.png")
                assert_clean_console(errors, "Sales Cycle screen")
                page.close()

                page, errors = new_page(browser, 1280, 800, runtime_script("ready"))
                ready = page.locator("[data-testid='runtime-status-ready']")
                ready.wait_for()
                assert ready.get_attribute("role") == "status"
                assert ready.get_attribute("aria-live") == "polite"
                assert page.locator("html").get_attribute("lang") == "ar-DZ"
                assert page.locator("html").get_attribute("dir") == "rtl"
                assert "0005" in ready.inner_text()
                assert_runtime_call_contract(page, 1)
                assert_command_bar_visible(page, "Arabic runtime ready 1280x800")
                assert_runtime_status_visible(page, "Arabic runtime ready 1280x800")
                assert_runtime_primary_unclipped(page, "Arabic runtime ready 1280x800")
                assert_workspace_labels_visible(page, "Arabic runtime ready rail 1280x800")
                assert_no_page_overflow(page, "Arabic runtime ready 1280x800")
                axe_summaries["phase-04-ar-runtime-ready"] = run_axe(
                    page, "phase-04-ar-runtime-ready"
                )
                screenshot(page, "phase-04-ar-runtime-ready.png")
                page.set_viewport_size({"width": 1024, "height": 640})
                assert_command_bar_visible(page, "Arabic runtime ready 1024x640")
                assert_runtime_status_visible(page, "Arabic runtime ready 1024x640")
                assert_runtime_primary_unclipped(page, "Arabic runtime ready 1024x640")
                assert_workspace_labels_visible(page, "Arabic runtime ready rail 1024x640")
                assert_no_page_overflow(page, "Arabic runtime ready 1024x640")
                assert_runtime_call_contract(page, 1)
                assert_clean_console(errors, "Arabic runtime ready")
                page.close()

                page, errors = new_page(browser, 1280, 800, runtime_script("ready"))
                page.locator("[data-testid='runtime-status-ready']").wait_for()
                page.locator("[data-testid='language-switch']").click()
                page.locator("text=Données locales prêtes").wait_for()
                assert page.locator("html").get_attribute("lang") == "fr-DZ"
                assert page.locator("html").get_attribute("dir") == "ltr"
                assert_runtime_call_contract(page, 1)
                assert_command_bar_visible(page, "French runtime ready 1280x800")
                assert_runtime_status_visible(page, "French runtime ready 1280x800")
                assert_runtime_primary_unclipped(page, "French runtime ready 1280x800")
                assert_workspace_labels_visible(page, "French runtime ready rail 1280x800")
                assert_no_page_overflow(page, "French runtime ready 1280x800")
                axe_summaries["phase-04-fr-runtime-ready"] = run_axe(
                    page, "phase-04-fr-runtime-ready"
                )
                screenshot(page, "phase-04-fr-runtime-ready.png")
                page.set_viewport_size({"width": 1024, "height": 640})
                assert_command_bar_visible(page, "French runtime ready 1024x640")
                assert_runtime_status_visible(page, "French runtime ready 1024x640")
                assert_runtime_primary_unclipped(page, "French runtime ready 1024x640")
                assert_workspace_labels_visible(page, "French runtime ready rail 1024x640")
                assert_no_page_overflow(page, "French runtime ready 1024x640")
                assert_runtime_call_contract(page, 1)
                assert_clean_console(errors, "French runtime ready")
                page.close()

                page, errors = new_page(browser, 1024, 640, runtime_script("error-retry"))
                notice = page.locator("[data-testid='runtime-error-notice']")
                notice.wait_for()
                body_text = page.locator("body").inner_text()
                assert "posman.sqlite3" not in body_text
                assert "SELECT" not in body_text
                assert "C:\\Users" not in body_text
                assert notice.get_attribute("data-error-code") == "RUNTIME_STATUS_UNAVAILABLE"
                assert_command_bar_visible(page, "Arabic runtime error 1024x640")
                assert_runtime_primary_unclipped(page, "Arabic runtime error 1024x640")
                assert_workspace_labels_visible(page, "Arabic runtime error rail")
                assert_no_page_overflow(page, "Arabic runtime error 1024x640")
                axe_summaries["phase-04-ar-runtime-error"] = run_axe(
                    page, "phase-04-ar-runtime-error"
                )
                retry = page.locator("[data-testid='runtime-retry']")
                retry.focus()
                page.keyboard.press("Enter")
                retrying = page.locator("[data-testid='runtime-retrying-notice']")
                retrying.wait_for()
                assert retrying.locator("button").is_disabled()
                page.locator("[data-testid='runtime-status-ready']").wait_for()
                assert_runtime_call_contract(page, 2)
                assert_runtime_status_visible(page, "Arabic runtime retry ready 1024x640")
                assert_runtime_primary_unclipped(page, "Arabic runtime retry ready 1024x640")
                assert_no_page_overflow(page, "Arabic runtime retry ready 1024x640")
                axe_summaries["phase-04-ar-runtime-retry"] = run_axe(
                    page, "phase-04-ar-runtime-retry"
                )
                screenshot(page, "phase-04-ar-runtime-retry.png")
                assert_clean_console(errors, "Arabic error and keyboard retry")
                page.close()

                click_page, click_errors = new_page(
                    browser, 1024, 640, runtime_script("error-retry")
                )
                click_page.locator("[data-testid='runtime-error-notice']").wait_for()
                click_page.locator("[data-testid='runtime-retry']").click()
                click_page.locator("[data-testid='runtime-status-ready']").wait_for()
                assert_runtime_call_contract(click_page, 2)
                assert_runtime_primary_unclipped(
                    click_page, "Arabic pointer retry ready 1024x640"
                )
                assert_clean_console(click_errors, "Arabic error and pointer retry")
                click_page.close()

                page, errors = new_page(browser, 1024, 640)
                page.locator("[data-testid='runtime-status-preview']").wait_for()
                page.locator("[data-testid='language-switch']").click()
                page.locator("text=Aperçu de l’interface").wait_for()
                assert page.locator("html").get_attribute("lang") == "fr-DZ"
                assert page.locator("html").get_attribute("dir") == "ltr"
                assert page.locator("[data-testid='runtime-status-ready']").count() == 0
                assert_command_bar_visible(page, "French runtime preview 1024x640")
                assert_runtime_status_visible(page, "French runtime preview 1024x640")
                assert_runtime_primary_unclipped(page, "French runtime preview 1024x640")
                assert_workspace_labels_visible(page, "French runtime preview rail")
                assert_no_page_overflow(page, "French runtime preview 1024x640")
                axe_summaries["phase-04-fr-runtime-preview"] = run_axe(
                    page, "phase-04-fr-runtime-preview"
                )
                screenshot(page, "phase-04-fr-runtime-preview.png")
                assert_clean_console(errors, "French runtime preview")
                page.close()

                page, errors = new_page(browser, 1024, 640, runtime_script("malformed"))
                page.locator("[data-testid='runtime-error-notice']").wait_for()
                assert page.locator("[data-testid='runtime-status-ready']").count() == 0
                assert page.locator("[data-testid='runtime-error-notice']").get_attribute(
                    "data-error-code"
                ) == "RUNTIME_STATUS_INVALID_RESPONSE"
                assert_runtime_call_contract(page, 1)
                assert_runtime_primary_unclipped(page, "Malformed runtime response 1024x640")
                assert_no_page_overflow(page, "Malformed runtime response 1024x640")
                axe_summaries["phase-04-malformed-response"] = run_axe(
                    page, "phase-04-malformed-response"
                )
                assert_clean_console(errors, "Malformed runtime response")
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
