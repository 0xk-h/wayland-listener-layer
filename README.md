# wldrop

A lightweight Wayland drag-and-drop daemon for Hyprland (and other wlr-based compositors) with near-zero overhead. It runs silently in the background as a layer surface, intercepts any file drop anywhere on the desktop, and hands the dropped file path off to a user-defined shell script — nothing more, nothing less.

**~0% CPU · ~10 MB RAM · fully transparent · always listening**

```
PID    USER   PRI  NI  VIRT   RES   SHR  S   CPU%  MEM%  TIME+    Command
81457  hunter  20   0  12020  10948  200  S   0.0   0.1   0:00.01  wldrop
```

---

## Installation

### Linux x86_64 — one-liner

```bash
bash install.sh
```

The script detects a local `wldrop` binary next to itself (post-build) and installs it to `/usr/local/bin`. If no local binary is found it downloads the latest release. `sudo` is invoked automatically only if `/usr/local/bin` isn't writable by your user.

### Build from source

```bash
git clone <repo>
cd wldrop
cargo build --release
# binary is at target/release/wldrop
bash install.sh          # installs from the local build
```

---

## How it works

wldrop places a **fully transparent, full-screen layer surface** at `Layer::Bottom` — above your wallpaper but below every application window. It never steals focus, never appears visually, and never polls. It simply sits in the Wayland event loop waiting for `wl_data_device` drag-and-drop events.

```
swww / wallpaper      →  Layer::Background
wldrop                →  Layer::Bottom   ← invisible, full-screen, always listening
app windows / bars    →  Layer::Top / default
```

When you drop a file onto the bare desktop:

1. The compositor routes the `Drop` event to wldrop's surface
2. wldrop reads the `text/uri-list` MIME payload and decodes the `file://` URI
3. Your script at `~/.config/wldrop/wldrop-exec.sh` is called with the file's absolute path as `$1`

Because it's purely event-driven with no timers or polling, CPU usage is effectively zero at rest and RAM stays around 10 MB for the lifetime of the process.

---

## Configuration

Create `~/.config/wldrop/wldrop-exec.sh` — this is the only configuration wldrop needs.

```bash
mkdir -p ~/.config/wldrop
cat > ~/.config/wldrop/wldrop-exec.sh << 'EOF'
#!/usr/bin/env bash

FILE="$1"

# open with default application
xdg-open "$FILE"

# --- other ideas ---
# cp "$FILE" ~/Desktop/
# mpv "$FILE"
# notify-send "Dropped" "$FILE"
EOF
chmod +x ~/.config/wldrop/wldrop-exec.sh
```

`$1` is the fully resolved local path (percent-encoding decoded, `file://` prefix stripped), e.g. `/home/hunter/Downloads/photo.png`.

If the script is missing, wldrop prints to stderr and exits gracefully:

```
~/.config/wldrop/wldrop-exec.sh file not present. Dropped file: /home/hunter/photo.png
```

---

## Autostart

Add to `~/.config/hypr/hyprland.conf`:

```ini
exec-once = wldrop
```

For other wlr compositors, use the equivalent autostart mechanism.

---

## Project structure

```
src/
├── main.rs       # Wayland init, layer surface setup, dual-roundtrip, event loop
├── app.rs        # App state (seat, shm pool, data device, pending offer)
├── handlers.rs   # SCT delegates — seat, compositor, shm, layer shell
├── dnd.rs        # wl_data_device + wl_data_offer dispatch — core DnD logic
└── utils.rs      # Unix pipe helper, URI percent-decoder, script runner
install.sh        # x86_64 installer — copies binary to /usr/local/bin
```

---

## Dependencies

| Crate                                | Role                                                      |
| ------------------------------------ | --------------------------------------------------------- |
| `smithay-client-toolkit`             | Layer shell, seat, shm — high-level Wayland abstractions  |
| `wayland-client`                     | Low-level Wayland protocol dispatch                       |
| `calloop` + `calloop-wayland-source` | Event loop                                                |
| `libc`                               | Raw `pipe()` for reading DnD payloads from the compositor |

---

## Caveats

- **Drops onto app windows are not intercepted.** If you drag a file onto a running application (file manager, browser, etc.) that handles DnD itself, that app takes the drop — wldrop only fires for drops that land on the bare desktop.
- **Single seat.** wldrop uses the first seat reported by the compositor. Multi-seat setups are untested.
- Requires `wlr-layer-shell-unstable-v1` — supported by Hyprland, sway, and most wlr-based compositors.
