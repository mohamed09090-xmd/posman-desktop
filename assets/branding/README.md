# POSMAN application mark

`posman-app-icon.png` is the approved POSMAN v1 application mark normalized for
desktop use. It keeps the original ledger-shaped geometry, uses the single
brand colour `#1F5A45`, has a transparent background, and reserves a safe area
so the mark remains legible at Windows taskbar and shortcut sizes.

The Windows PNG and ICO derivatives in `src-tauri/icons/` are generated from
this master with the pinned Tauri CLI:

```text
npx tauri icon assets/branding/posman-app-icon.png --output <temporary-directory>
```

Only the Windows/Tauri derivatives referenced by `tauri.conf.json` are kept in
the repository. Do not redraw, stretch, rotate, shadow, gradient-fill, or place
the mark on a permanent coloured tile.
