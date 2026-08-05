# mokey

An open-source keyboard-driven mouse replacement. An alternative to [mouseless](https://github.com/milgra/mouseless) that combines a grid-zoom (number-input) UX with an optional Vim mode.

- **Grid zoom**: press the trigger key → the screen splits into a numbered grid → type a number to zoom in, repeat → Enter/click
- **Vim mode (off by default)**: `hjkl` movement, `m` left click, `,.` scroll, `v` drag, digit repeat-count prefix
- **Beginner-friendly**: you just type a number right after the trigger; Vim bindings are opt-in from settings
- **Lightweight**: Rust + egui, no webview, single binary

## Supported platforms

| Platform | Status |
| --- | --- |
| Windows 11 (x64) | ✅ In development (Phase 1) |
| Hyprland (Wayland) | ⏳ Phase 2 |
| KDE Plasma (Wayland) | ⏳ Phase 3 |
| GNOME (Wayland) | ⏳ Phase 3 |

X11 and macOS are not planned.

## Build

```sh
cargo build --release
```

mokey uses low-level keyboard/mouse APIs (rdev, enigo, windows-sys) on Windows.
It works without administrator privileges in most cases, though some apps may
restrict global input hooks.

## Usage

1. Run `mokey` (idles in the background)
2. Press `Ctrl+Alt+Space` → a numbered grid appears over the screen
3. Type the target cell number, zoom in as needed, then `Enter` to click, `Backspace` to zoom out, `Esc` to cancel
4. Settings window: `Ctrl+Alt+S`

Config file: `%USERPROFILE%\.config\mokey\config.toml`

```toml
[general]
trigger_hotkey = "Ctrl+Alt+Space"
settings_hotkey = "Ctrl+Alt+S"
grid_size = 3
max_depth = 4
auto_click = true
overlay_bg_opacity = 0.45
move_step = 10
move_fast_step = 100

[vim]
enabled = false
```

## Architecture

- **`mokey-core`**: platform-agnostic logic (grid math, session state, key parsing, config schema)
- **`mokey-backend`**: platform-specific mouse control (enigo), global key hooks (rdev), monitor/DPI enumeration (windows-sys). Wayland plans: layer-shell/hyprctl/ydotool/KGlobalAccel
- **`mokey-app`**: egui-based HUD overlay + settings window (eframe)

## Roadmap

- Phase 1: Windows MVP (current)
- Phase 2: Hyprland support
- Phase 3: KDE Plasma + GNOME support
- Phase 4: More features (gestures, coordinate bookmarks, etc.)

## License

MIT. See [LICENSE](LICENSE).
