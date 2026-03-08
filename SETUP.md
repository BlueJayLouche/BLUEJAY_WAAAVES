# RustJay Waaaves Setup Guide

High-performance VJ application in Rust with wgpu shaders and Syphon support.

## Prerequisites

### 1. Rust Toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Local Syphon Framework

The Syphon framework is shared across the workspace:

```
../crates/syphon/syphon-lib/Syphon.framework
```

No system installation required! The build script automatically links this local copy.

### 3. Build Tools

```bash
# Standard build tools should be sufficient
# macOS Command Line Tools
xcode-select --install
```

## Quick Start

```bash
# Clone and enter directory
cd rustjay_waaaves

# Build with Syphon support
./run.sh

# Or manually:
cargo run --features ipc-syphon
```

## Development Workflow

### Using run.sh

```bash
./run.sh [OPTIONS]

Options:
  --release       Build optimized version
  --no-syphon     Run without Syphon
  --no-check      Skip framework check
  --help          Show help
```

### Manual Build

```bash
# Debug build
cargo build --features ipc-syphon

# Release build (optimized)
cargo build --release --features ipc-syphon

# Run
cargo run --release --features ipc-syphon
```

## Syphon Usage

### Output (Send to Resolume, MadMapper, etc.)

```rust
use crate::output::syphon_sender::SyphonWgpuSender;

// Create output
let mut output = SyphonWgpuSender::new(
    "RustJay Output",
    &device,
    &queue,
    1920,
    1080
)?;

// Each frame, publish texture
output.publish(&render_texture, &device, &queue);
```

### Input (Receive from other apps)

```rust
use crate::input::syphon_input::{
    SyphonInputReceiver,
    SyphonDiscovery,
    SyphonInputIntegration
};

// Simple receiver
let mut receiver = SyphonInputReceiver::new();
receiver.connect("Resolume Arena")?;

// Receive frames in background
while let Some(frame) = receiver.get_latest_frame() {
    let bgra_data = frame.data;
    // Upload to GPU texture
}

// Or use high-level integration
let mut input = SyphonInputIntegration::new();
input.refresh_servers();        // Discover available servers
input.connect("Server Name")?;  // Connect to specific server
input.update();                 // Poll for new frames
```

## Troubleshooting

### "Local Syphon.framework not found"

```bash
# Ensure framework exists
ls ../crates/syphon/syphon-lib/Syphon.framework/

# If missing, copy it:
cp -R ~/Downloads/Syphon.framework ../crates/syphon/syphon-lib/
```

### Crash when connecting to Syphon input

**Important:** This was a known issue with autoreleasepool. Fixed in latest syphon-core.

If you still see crashes:

```bash
# Rebuild syphon-core
cd ../crates/syphon/syphon-core
cargo build

# Then rebuild app
cd /path/to/rustjay_waaaves
cargo clean && cargo build --features ipc-syphon
```

### "No Syphon servers found"

Servers take a moment to announce themselves. The app will retry automatically.

### High CPU usage

This is normal during shader compilation. Runtime CPU usage should be low.

## Creating App Bundle

For distribution:

```bash
# Install cargo-bundle
cargo install cargo-bundle

# Create .app bundle
cargo bundle --release

# Result:
# target/release/bundle/osx/RustJay\ Waaaves.app
```

## Project Structure

```
rustjay_waaaves/
├── Cargo.toml          # Dependencies & bundle config
├── build.rs            # Links local Syphon framework
├── run.sh              # Development helper
├── SETUP.md            # This file
├── src/
│   ├── input/
│   │   ├── mod.rs
│   │   └── syphon_input.rs    # Syphon input receiver
│   ├── output/
│   │   ├── mod.rs
│   │   └── syphon_sender.rs   # Syphon output
│   └── ipc/
│       └── syphon.rs          # Re-exports & utilities
└── ...
```

## GPU Compatibility

Check available GPUs:

```rust
use crate::ipc::syphon::check_gpu_compatibility;

match check_gpu_compatibility() {
    Ok(report) => println!("{}", report),
    Err(e) => println!("GPU check failed: {}", e),
}
```

Output example:
```
Multi-GPU system (2 GPUs):
  - Apple M1 Pro (default=true, high-performance=true)
  - Apple M1 (default=false, high-performance=false)
```

## Features

- ✅ Real-time shader pipeline (wgpu)
- ✅ Zero-copy Syphon output
- ✅ Syphon input (receive from other apps)
- ✅ NDI input/output
- ✅ Webcam input
- ✅ MIDI control
- ✅ OSC support
- ✅ Audio reactivity

## Links

- [Syphon Crate Documentation](../crates/syphon/README.md)
- [Syphon Troubleshooting](../crates/syphon/TROUBLESHOOTING.md)
- [Migration Guide](../crates/syphon/MIGRATION_GUIDE.md)
- [Syphon Framework](https://github.com/Syphon/Syphon-Framework)
