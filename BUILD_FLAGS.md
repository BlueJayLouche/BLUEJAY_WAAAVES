# Build Flags Guide

## Platform-Specific Builds

### macOS (with Syphon support)
```bash
cargo run --features ipc-syphon
```

### Windows (with Spout support)
```bash
cargo run --features ipc-spout
```

### Linux (with V4L2 support)
```bash
cargo run --features ipc-v4l2
```

### Generic (webcam + NDI only, no IPC)
```bash
cargo run
```

### All Platforms (everything enabled)
```bash
cargo run --features ipc-all
```

## Feature Flags

| Flag | Platform | What it enables |
|------|----------|-----------------|
| `webcam` | All | Webcam capture via nokhwa |
| `ipc-syphon` | macOS only | Syphon video sharing |
| `ipc-spout` | Windows only | Spout video sharing |
| `ipc-v4l2` | Linux only | V4L2 video devices |
| `ipc-all` | All | Enables all IPC (for testing) |

## Why This Matters

**Without platform-specific flags:**
- macOS: Compiles syphon-core, spout deps, AND v4l2 deps (wasted compile time)
- Linux: Compiles syphon-core and winapi (completely unused)

**With platform-specific flags:**
- Each platform only compiles what it actually uses
- Faster builds
- Smaller binaries

## Current Default

Currently `default = ["webcam"]` - Syphon is NOT enabled by default anymore.
You must explicitly add `--features ipc-syphon` on macOS.
