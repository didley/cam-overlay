# Cam Overlay

A minimal cross-platform app that displays a webcam preview as a borderless, always-on-top overlay for use during screen recording or presentations.

No recording — just a live preview with zoom, shape, flip, and scale controls via right-click menu.

## Features

- Live webcam preview (V4L2 on Linux, AVFoundation on macOS)
- Circle or rounded rectangle shape clipping
- 1×, 1.5×, and 2× zoom
- Scale modes: Crop, Fit, Stretch
- Horizontal mirror/flip
- Double-click to expand fullscreen
- Drag to move, edge-drag to resize
- Right-click context menu
- Always-on-top window
- Settings persist across restarts

## Build

Requires only a Rust stable toolchain — no system dev packages needed.

```sh
cargo build --release
```

### Run

```sh
cargo run --release
```

## Requirements

- Rust stable toolchain
- Linux: a V4L2-compatible webcam
- macOS: camera access permission (prompted on first run)

## Tech Stack

- **winit** — cross-platform windowing
- **wgpu** — GPU-accelerated rendering
- **nokhwa** — webcam capture (V4L2 / AVFoundation)
- **confy** — cross-platform settings persistence

## License

GPL-3.0-or-later
