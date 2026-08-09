#!/usr/bin/env python3
"""Verify POSMAN v1 distribution, branding, and data-retention contracts."""
from __future__ import annotations

import hashlib
import json
import re
import struct
import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VERSION = "1.0.0"
FROZEN_MIGRATIONS = {
    "0001_system_company_security.sql": "af2d8df4e6aadb0333a5b5e7e893d85da0527e4c286462d1fb1c1861fa272735",
    "0002_reference_catalog_partners.sql": "f7aab1bb8f8784624cadb4cc9d1cb7e6dde56cad1cbffffa4da90a8e48e7b715",
    "0003_commerce_inventory.sql": "093aa71fe7e8ba58b6b487a7c578cd39c353b3225783ce87cabf6a2e8a111d39",
    "0004_accounting_documents_audit.sql": "c7d9ac5e194f1c1f47cd4d37f691218635fc6a98b23dd9afbb5a541538f7d99e",
    "0005_setup_security_reference_data.sql": "10eab9cadd76adbefa60ad9891b737549948d06d5fb8ea8437ac160f7d91127f",
    "0006_accounting_payments_hardening.sql": "08763076ce7cbd77e585bf06b10bc856e7b8f02193484b1db974db95143cebd0",
    "0007_phase09_documents_reports_audit_backup.sql": "f22f220ccf6ae2f85f0be85ae018b9f5760e725e9e88c6b2b87c37598424eb90",
}
REQUIRED_DOCUMENTS = (
    "CHANGELOG.md",
    "SECURITY.md",
    "docs/release/POSMAN-1.0.0-RELEASE-NOTES.md",
    "docs/release/WINDOWS-INSTALLATION.md",
    "docs/release/CODE-SIGNING.md",
    "docs/architecture/phase-10-distribution-hardening.md",
    "docs/execution/POSMAN-PHASE-10-EXECUTION-PACK.md",
    "docs/PHASE-10-REPORT.md",
)


def fail(message: str) -> None:
    raise SystemExit(f"PHASE10 VERIFY FAILED: {message}")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(relative: str) -> dict:
    path = ROOT / relative
    if not path.is_file():
        fail(f"missing JSON file: {relative}")
    return json.loads(path.read_text(encoding="utf-8"))


def png_rgba(path: Path) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        fail(f"invalid PNG signature: {path.relative_to(ROOT)}")
    position = 8
    width = height = bit_depth = colour_type = interlace = None
    compressed = bytearray()
    while position < len(data):
        length = struct.unpack(">I", data[position : position + 4])[0]
        kind = data[position + 4 : position + 8]
        payload = data[position + 8 : position + 8 + length]
        position += 12 + length
        if kind == b"IHDR":
            width, height, bit_depth, colour_type, _, _, interlace = struct.unpack(
                ">IIBBBBB", payload
            )
        elif kind == b"IDAT":
            compressed.extend(payload)
        elif kind == b"IEND":
            break
    if None in (width, height, bit_depth, colour_type, interlace):
        fail(f"missing PNG header: {path.relative_to(ROOT)}")
    if bit_depth != 8 or colour_type != 6 or interlace != 0:
        fail(f"PNG must be non-interlaced 8-bit RGBA: {path.relative_to(ROOT)}")

    encoded = zlib.decompress(bytes(compressed))
    stride = width * 4
    rows: list[bytearray] = []
    previous = bytearray(stride)
    cursor = 0
    for _ in range(height):
        filter_kind = encoded[cursor]
        cursor += 1
        source = encoded[cursor : cursor + stride]
        cursor += stride
        row = bytearray(stride)
        for index, value in enumerate(source):
            left = row[index - 4] if index >= 4 else 0
            above = previous[index]
            upper_left = previous[index - 4] if index >= 4 else 0
            if filter_kind == 0:
                prediction = 0
            elif filter_kind == 1:
                prediction = left
            elif filter_kind == 2:
                prediction = above
            elif filter_kind == 3:
                prediction = (left + above) // 2
            elif filter_kind == 4:
                estimate = left + above - upper_left
                distances = (
                    abs(estimate - left),
                    abs(estimate - above),
                    abs(estimate - upper_left),
                )
                prediction = (left, above, upper_left)[distances.index(min(distances))]
            else:
                fail(f"unsupported PNG filter {filter_kind}: {path.relative_to(ROOT)}")
            row[index] = (value + prediction) & 0xFF
        rows.append(row)
        previous = row
    return width, height, b"".join(rows)


def verify_branding() -> None:
    expected_sizes = {
        "assets/branding/posman-app-icon.png": (1024, 1024),
        "src-tauri/icons/32x32.png": (32, 32),
        "src-tauri/icons/128x128.png": (128, 128),
        "src-tauri/icons/128x128@2x.png": (256, 256),
    }
    for relative, expected in expected_sizes.items():
        width, height, pixels = png_rgba(ROOT / relative)
        if (width, height) != expected:
            fail(f"unexpected icon size for {relative}: {(width, height)}")
        alphas = pixels[3::4]
        if min(alphas) != 0 or max(alphas) != 255:
            fail(f"icon must contain transparent and opaque pixels: {relative}")
        if any(alphas[index] != 0 for index in (0, width - 1, width * (height - 1), width * height - 1)):
            fail(f"icon corners must be transparent: {relative}")
    _, _, master = png_rgba(ROOT / "assets/branding/posman-app-icon.png")
    visible = [master[index : index + 4] for index in range(0, len(master), 4) if master[index + 3] > 240]
    if not visible or any(
        any(abs(channel - expected) > 2 for channel, expected in zip(pixel[:3], (31, 90, 69)))
        for pixel in visible
    ):
        fail("opaque master-icon pixels must use brand colour #1F5A45")
    ico = (ROOT / "src-tauri/icons/icon.ico").read_bytes()
    if len(ico) < 6 or ico[:4] != b"\x00\x00\x01\x00" or struct.unpack("<H", ico[4:6])[0] < 6:
        fail("Windows icon must be a multi-resolution ICO")


def verify_versions() -> None:
    package = read_json("package.json")
    package_lock = read_json("package-lock.json")
    tauri = read_json("src-tauri/tauri.conf.json")
    cargo = (ROOT / "src-tauri/Cargo.toml").read_text(encoding="utf-8")
    cargo_lock = (ROOT / "src-tauri/Cargo.lock").read_text(encoding="utf-8")
    versions = {
        "package.json": package.get("version"),
        "package-lock.json": package_lock.get("version"),
        "package-lock root": package_lock.get("packages", {}).get("", {}).get("version"),
        "tauri.conf.json": tauri.get("version"),
        "Cargo.toml": re.search(r'^version = "([^"]+)"', cargo, re.MULTILINE).group(1),
    }
    root_lock = re.search(
        r'\[\[package\]\]\nname = "posman-desktop"\nversion = "([^"]+)"', cargo_lock
    )
    versions["Cargo.lock"] = root_lock.group(1) if root_lock else None
    wrong = {name: value for name, value in versions.items() if value != VERSION}
    if wrong:
        fail(f"v1 version parity failed: {wrong}")


def verify_installer_contract() -> None:
    tauri = read_json("src-tauri/tauri.conf.json")
    bundle = tauri.get("bundle", {})
    windows = bundle.get("windows", {})
    nsis = windows.get("nsis", {})
    if bundle.get("targets") != ["nsis"]:
        fail("production bundle target must be NSIS only")
    if windows.get("webviewInstallMode") != {"type": "offlineInstaller", "silent": True}:
        fail("production bundle must embed the silent WebView2 offline installer")
    if windows.get("allowDowngrades") is not False:
        fail("installer downgrades must be blocked")
    if nsis.get("installMode") != "perMachine":
        fail("NSIS must install binaries per-machine outside the POSMAN data root")
    if nsis.get("compression") != "lzma" or nsis.get("template") != "installer.nsi":
        fail("NSIS must use the pinned high-dictionary LZMA template")
    if nsis.get("languages") != ["Arabic", "French", "English"]:
        fail("installer language order must be Arabic, French, English")
    if nsis.get("displayLanguageSelector") is not True:
        fail("installer language selector must be enabled")
    if nsis.get("installerIcon") != "icons/icon.ico" or nsis.get("uninstallerIcon") != "icons/icon.ico":
        fail("installer and uninstaller must use the approved POSMAN icon")
    installer_template = (ROOT / "src-tauri/installer.nsi").read_text(encoding="utf-8")
    for marker in (
        "Tauri v2.11.5 installer.nsi",
        "SetCompressor /SOLID \"{{compression}}\"",
        "SetCompressorDictSize 128",
        'File "/oname=$TEMP\\MicrosoftEdgeWebView2RuntimeInstaller.exe" "${WEBVIEW2INSTALLERPATH}"',
    ):
        if marker not in installer_template:
            fail(f"pinned offline NSIS template contract missing: {marker}")
    for signing_key in ("certificateThumbprint", "timestampUrl", "signCommand"):
        if windows.get(signing_key):
            fail(f"repository configuration must not contain signing material: {signing_key}")

    library = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
    if '.local_data_dir()' not in library or '.join("POSMAN")' not in library:
        fail("runtime data root must remain %LOCALAPPDATA%/POSMAN")
    fixture = read_json("src-tauri/tauri.phase10-upgrade-fixture.conf.json")
    if fixture.get("version") != "0.9.0":
        fail("upgrade fixture must model a lower v0.9.0 install")
    if fixture.get("bundle", {}).get("windows", {}).get("webviewInstallMode") != {
        "type": "offlineInstaller",
        "silent": True,
    }:
        fail("upgrade fixture must retain the valid offline WebView2 installer contract")


def main() -> int:
    for name, expected in FROZEN_MIGRATIONS.items():
        path = ROOT / "database/migrations" / name
        if not path.is_file() or digest(path) != expected:
            fail(f"accepted migration changed: {name}")
    if len(list((ROOT / "database/migrations").glob("*.sql"))) != 7:
        fail("PHASE 10 must not add a database migration")
    verify_versions()
    verify_installer_contract()
    verify_branding()
    for relative in REQUIRED_DOCUMENTS:
        if not (ROOT / relative).is_file():
            fail(f"required release document missing: {relative}")
    workflow = (ROOT / ".github/workflows/phase10-ci.yml").read_text(encoding="utf-8")
    for marker in (
        "permissions:\n  contents: read",
        "POSMAN-Setup-Offline.exe",
        "test_installer_lifecycle.ps1",
        "phase-10-release-evidence",
        "posman-v1.0.0-windows-offline",
    ):
        if marker not in workflow:
            fail(f"PHASE 10 workflow contract missing: {marker}")
    print(
        "PHASE10 VERIFY PASS: version 1.0.0; transparent approved branding; "
        "offline NSIS per-machine install; protected LocalAppData; migrations 0001-0007 frozen"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
