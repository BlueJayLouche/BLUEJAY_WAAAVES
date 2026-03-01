# Feature Wishlist

> Last Updated: 28 Feb 2026
> 
> **Note:** This document contains planned features for the RustJay Waaaves VJ application. Each feature includes an actionable prompt that can be given to an AI assistant or developer for implementation.

---

## ⚠️ Pre-Implementation Checklist

Before implementing any new features:
- [ ] Create a git branch: `git checkout -b feature/name`
- [ ] Run tests: `cargo test`
- [ ] Verify current build compiles: `cargo check`
- [ ] Update this document to mark feature as "In Progress"

---

## GUI

### 1. All Tabs as Optional Pop-out, Resizable Windows
**Status:** ⏳ Not Started

**Description:**
Convert the current single-window ImGui interface into a multi-window dockable workspace. Each block tab (Block 1, Block 2, Block 3, Macros, Inputs, Settings) should be detachable into its own floating window that can be resized, moved, and docked independently. This allows users to create custom workspace layouts based on their workflow (e.g., having Block 1 and Block 2 visible simultaneously on dual monitors).

**Actionable Prompt:**
> Implement a dockable multi-window GUI system using `imgui-docking` or similar. Convert the current single-window tabbed interface so that each main tab (Block 1, Block 2, Block 3, Macros, Inputs, Settings) can be popped out into separate, resizable windows. Users should be able to drag tabs to undock them, resize each window independently, and dock them back into a main workspace. Ensure the shared state (`Arc<Mutex<SharedState>>`) remains thread-safe across multiple windows. Add a "Window" menu with options to show/hide each panel and "Reset to Default Layout".

**Technical Notes:**
- Use `imgui-docking` branch or feature flag
- Each panel needs its own `imgui::Context` or use shared context with docking
- Window positions stored in pixels - handle DPI scaling
- Consider platform-specific window decorations

**References:**
- See `src/gui/mod.rs` - current single-window implementation
- winit multi-window example for event loop handling

---

### 2. Layout Independently Savable to Persist Between Restarts
**Status:** ⏳ Not Started

**Description:**
Save and restore custom GUI layouts including window positions, sizes, docked/undocked states, and which tabs are visible. This should be saved to a user config file (separate from presets) and automatically loaded on application startup. Include the control window's current size (currently forced fullscreen).

**Actionable Prompt:**
> Implement layout persistence for the multi-window GUI. Create a new config file (e.g., `layout.toml` in the config directory) that stores: window positions (x, y), window sizes (width, height), docked/floating state, visibility of each panel/tab, and monitor assignment for multi-monitor setups. Save layout automatically when windows are moved/resized (debounced), or add a "Save Layout" button. Load and apply the saved layout on application startup before windows are created. Handle cases where saved monitor configurations are no longer available (fallback to primary monitor with clamped coordinates).

**Technical Notes:**
- Use existing `src/config/mod.rs` infrastructure
- Add `LayoutConfig` struct with serde derive
- Save to `~/.config/rustjay_waaaves/layout.toml` (platform appropriate)
- Handle missing monitors: check if position is within any available monitor bounds

**Example Config Structure:**
```toml
[layout]
version = "1.0"

[layout.windows.control]
position = [100, 100]
size = [1200, 800]
visible = true
maximized = false

[layout.windows.block1_panel]
position = [1300, 100]
size = [400, 600]
visible = true
floating = true

[layout.windows.block2_panel]
visible = false  # User closed this panel
```

---

### 3. Scale - Text is Very Small
**Status:** ✅ Completed (28 Feb 2026)

**Description:**
The ImGui interface text is too small on high-DPI displays (Retina Macs, 4K monitors). Add UI scaling controls to increase font size globally with a slider.

**Actionable Prompt:**
> Add UI scaling/font size controls to the Settings tab. Store a `ui_scale` factor (0.5x to 2.0x, default 1.0) in config. Scale all ImGui fonts and spacing proportionally using `imgui.io_mut().font_global_scale`. Update the ImGui font atlas with scaled font sizes on startup and when scale changes. Ensure controls remain usable at all scale factors - test at 0.5x, 1.0x, 1.5x, and 2.0x. Consider auto-detecting display DPI on macOS/Windows for sensible defaults (check `window.scale_factor()` from winit).

**Technical Notes:**
- ImGui font atlas needs rebuild when scale changes
- May need multiple font sizes pre-loaded for performance
- Scale affects all `imgui::Drag`, `imgui::Text`, spacing, etc.
- Store scale in `AppConfig` and apply in `ControlGui::new()`

**Code Hint:**
```rust
// In gui initialization
let scale = config.ui_scale;
imgui.io_mut().font_global_scale = scale;
// Or rebuild font atlas with scaled size
```

---

### 4. Framerate/Resolution Control + Visualize FPS
**Status:** ⏳ Not Started

**Description:**
Add GUI controls to set the target framerate (affects animation smoothness and LFO timing) and display current FPS. This is separate from the output window framerate (see Main Output section).

**Actionable Prompt:**
> Add framerate controls to Settings tab: target FPS slider (24-144), VSync toggle, and FPS limiter enable/disable. Display current FPS in real-time as a small overlay in the corner of the control window (toggleable). Use a rolling average (last 60 frames) for smooth display. Store target FPS in config. Ensure LFO and tempo calculations use actual delta time rather than assuming fixed FPS for accuracy. Add a frame time graph (optional) showing recent frame times.

**Technical Notes:**
- Use `ControlFlow::WaitUntil` for frame limiting
- FPS counter: calculate from `Instant::now()` delta
- Rolling average: `fps = 0.9 * fps + 0.1 * current_fps`
- LFO engine already uses delta_time - verify this is working correctly

**Code Hint:**
```rust
// Frame limiting in event loop
let next_frame_time = last_frame_time + frame_duration;
*control_flow = ControlFlow::WaitUntil(next_frame_time);
```

---

### 5. Preview Window Improvements
**Status:** ⏳ Not Started

**Description:**
Reuse the Block 1 debug view system and add a color picker that can be assigned to keys (like in the original oF app where you could pick colors from the preview by clicking).

**Actionable Prompt:**
> Enhance the preview/debug view window. Reuse the existing Block 1 debug view infrastructure (`Block1DebugView` enum) to show: final output, individual stage outputs, and a color picker overlay. When color picker mode is active (toggle button), clicking on the preview window samples the pixel color under the cursor and assigns it to the currently selected key/color parameter. Add keyboard shortcuts to toggle between preview modes (1=final output, 2=block 1 only, 3=block 2 only, 4=wireframe/debug overlay). Display cursor coordinates and sampled RGB/HSB values in real-time. The color picker should work with FB1/FB2 key color values.

**Technical Notes:**
- Need to read back GPU texture data for color picking (expensive!)
- Alternative: maintain CPU-side copy of last frame, or sample at lower res
- Coordinate conversion: window pixels → UV coordinates → texture sample
- See existing debug view in `build_block1_panel()`

**References:**
- See `src/engine/blocks/block1.rs` - `Block1DebugView` enum
- oF app had eyedropper tool - check original implementation

---

## Main Output

### 6. Fullscreenable via Shift+F
**Status:** ⏳ Not Started

**Description:**
Add keyboard shortcut support to toggle the main output window between windowed and fullscreen modes. Shift+F is the standard shortcut from the original oF application.

**Actionable Prompt:**
> Add fullscreen toggle functionality to the main output window. Implement keyboard event handling in the winit event loop (in `main.rs` or engine) to detect Shift+F key combination. When triggered, toggle the output window between windowed and fullscreen modes using winit's `window.set_fullscreen()` API. Ensure the correct monitor is used (the one containing the output window). Persist the fullscreen state in the config and restore on restart if desired. Add visual feedback (e.g., brief on-screen notification) when fullscreen mode changes. Prevent the control window from also going fullscreen.

**Technical Notes:**
- Use `Fullscreen::Borderless(Some(monitor))` for best compatibility
- Or `Fullscreen::Exclusive` for dedicated fullscreen (less recommended)
- Store: `fullscreen: bool` and `fullscreen_monitor: Option<MonitorHandle>`
- Distinguish which window received the key event (control vs output)

**Code Hint:**
```rust
// In window event handler
WindowEvent::KeyboardInput { input, .. } => {
    if input.virtual_keycode == Some(VirtualKeyCode::F) 
        && input.modifiers.shift() {
        let fullscreen = if window.fullscreen().is_some() {
            None
        } else {
            Some(Fullscreen::Borderless(None))
        };
        window.set_fullscreen(fullscreen);
    }
}
```

---

### 7. Tunable Framerate Control - 24-60 (>60?) fps
**Status:** ⏳ Not Started

**Description:**
Control the rendering framerate of the output window independently from the control window. Allow setting anywhere from 24fps (cinematic) up to 60fps or higher for high-refresh displays (120Hz, 144Hz).

**Actionable Prompt:**
> Add output window framerate control to Settings tab. Add a slider from 24 to 144 FPS with "Unlimited" option. Implement frame rate limiting using winit's `ControlFlow::WaitUntil` or manual frame timing with `std::thread::sleep`. Display actual achieved FPS vs target in the GUI. Ensure frame timing is consistent for smooth LFO animations - use `Instant::elapsed()` for delta time calculations. Consider adaptive vsync options. Store target FPS separately for output and control windows. The output window should render independently even if control window is at lower FPS.

**Technical Notes:**
- Two windows = potentially two event loops or shared loop
- Current architecture: control window drives the loop
- May need separate render thread for output window
- Frame pacing is critical for smooth feedback visuals

**References:**
- See `src/engine/simple_engine.rs` for frame timing example
- Current implementation in `WgpuEngine::render()`

---

## Control

### 8. Full MIDI Mapping Capabilities
**Status:** ⏳ Not Started

**Description:**
Implement a MIDI learn/mapping system where users can click any parameter in the GUI and then trigger a MIDI controller (note, CC, or other message) to map that control. The mapping should persist and allow real-time control of parameters via MIDI hardware.

**Actionable Prompt:**
> Implement a comprehensive MIDI mapping system using the `midir` crate. Add a "MIDI Learn" toggle button in the GUI (perhaps in a new "MIDI" tab). When enabled, clicking any parameter puts it in "learning" state (visual highlight - green border or text). The next MIDI message received (CC, Note On, Pitch Bend) from any connected MIDI input device is mapped to that parameter. Store mappings in `HashMap<String, MidiMapping>` where MidiMapping contains: device_id, message_type, channel, controller/note number. Support range scaling (map MIDI 0-127 to param min-max). Add a MIDI mappings panel showing all active mappings with delete buttons. Save/load mappings to/from `midi_mappings.toml`. Handle multiple MIDI devices and hot-plugging. Real-time update: When MIDI message is received, immediately update the corresponding parameter in SharedState.

**Technical Notes:**
- Use `midir` crate for cross-platform MIDI
- MIDI input is asynchronous - use channel to send to main thread
- Consider MIDI output for feedback (LED updates)
- Support 14-bit CC (MSB + LSB) for high-resolution parameters

**Dependencies:**
```toml
[dependencies]
midir = "0.10"
```

**Example MIDI Config:**
```toml
[[mappings]]
param_id = "block1.fb1_mix_amount"
device = "Akai APC40"
message_type = "CC"
channel = 1
controller = 16
min_value = 0.0
max_value = 1.0
```

**References:**
- OF app MIDI implementation
- `midir` examples for input handling

---

### 9. OSC Address Expose - Toggleable - Hover Over Param to See OSC Address
**Status:** ⏳ Not Started

**Description:**
Add an optional feature that displays the OSC (Open Sound Control) address for each parameter when hovering over it. This allows users to know which OSC path to use for external control via OSC controllers (TouchOSC, Lemur, custom scripts, etc.).

**Actionable Prompt:**
> Implement OSC address tooltip display system. First, define a consistent OSC address scheme for all parameters (e.g., `/block1/ch1/x_displace`, `/block2/fb2/mix_amount`, `/block3/final/mix_amount`). Add a toggle button in the Settings tab: "Show OSC Addresses". When enabled, hovering over any parameter control should display a tooltip showing the full OSC address path for that parameter and its current value. The OSC path should follow a hierarchical structure: `/{block}/{section}/{parameter}`. Include a "Copy OSC Address" option in the right-click context menu for each parameter. This OSC scheme should match what would be used if implementing an OSC receiver for parameter control.

**Technical Notes:**
- OSC paths are strings - use consistent naming convention
- Match the internal Rust struct names for predictability
- Example: `/block1/ch1/x_displace` → `Block1Params.ch1_x_displace`
- Example: `/block3/matrix/r_to_g` → `Block3Params.matrix_mix_r_to_g`

**OSC Namespace Draft:**
```
/block1/ch1/{param}       - Channel 1 adjust params
/block1/ch2/{param}       - Channel 2 adjust params  
/block1/fb1/{param}       - FB1 params
/block1/ch2/mix/{param}   - CH2 mix & key params
/block2/input/{param}     - Block 2 input params
/block2/fb2/{param}       - FB2 params
/block3/block1/{param}    - Block 1 re-process
/block3/block2/{param}    - Block 2 re-process
/block3/matrix/{param}    - Matrix mixer params
/block3/final/{param}     - Final mix params
/lfo/{bank}/{param}       - LFO bank params
/global/bpm               - Global BPM
/global/tap_tempo         - Tap tempo trigger
```

**References:**
- TouchOSC layout editor for common patterns
- oF ofxOsc addon for existing implementation patterns

---

## Audio Reactivity/LFOs

### 10. Tap Tempo - Tap via Keypress (Shift+T)
**Status:** ⏳ Not Started

**Description:**
Add keyboard shortcut (Shift+T) to tap tempo globally, in addition to any GUI button. This allows quick tempo matching to music without using the mouse.

**Actionable Prompt:**
> Add global keyboard tap tempo functionality. Detect Shift+T key combination in the winit event loop, regardless of which window has focus. Feed this into the existing tap tempo system in `src/gui/mod.rs` (`handle_tap_tempo()` method). The tap should: record current time, calculate BPM from intervals after 2+ taps, reset calculation if gap > 2 seconds between taps, reset all LFO phases on each tap, and provide visual feedback (brief flash or status message). Ensure this works alongside the GUI tap button - both should use the same underlying tap tempo logic. Display the calculated BPM immediately after 2 taps.

**Technical Notes:**
- Reuse existing `tap_times: Vec<f64>` and `handle_tap_tempo()` logic
- Global hotkey: may need to register with OS for global shortcuts
- Or just ensure both windows check for the key
- Visual feedback: could flash the BPM display green momentarily

**Code Hint:**
```rust
// In event loop for both windows
WindowEvent::KeyboardInput { input, .. } => {
    if input.virtual_keycode == Some(VirtualKeyCode::T) 
        && input.modifiers.shift() {
        control_gui.handle_tap_tempo();
    }
}
```

**References:**
- See existing `handle_tap_tempo()` in `src/gui/mod.rs`
- See `src/core/lfo_engine.rs` for BPM calculations

---

### 11. Finalise Mappings - Missing
**Status:** 🔄 In Progress

**Description:**
Complete the LFO parameter mappings. The GUI lists many LFO parameters but some may not be fully implemented in the LFO engine or may have missing/incorrect parameter IDs.

**Actionable Prompt:**
> Audit and finalize all LFO parameter mappings. Verify every parameter in `Block1Params`, `Block2Params`, and `Block3Params` has a corresponding LFO entry if applicable. Check `src/core/lfo_engine.rs` - the `apply_lfos_to_block1/2/3()` functions - and ensure all parameters listed in the GUI LFO panels are handled in the match statements. Test each LFO assignment works correctly: modulation applies, amplitude scaling works, tempo sync functions. Ensure component fields (e.g., `_x`, `_y`, `_z`) stay synchronized with their parent Vec3/Vec4 fields when modulated. Fix any parameter IDs that don't match between GUI and LFO engine. Document the complete LFO parameter list in AGENTS.md for reference.

**Technical Notes:**
- Parameter IDs must match exactly between GUI and LFO engine
- Vec3 components need special handling: update both individual field and parent Vec3
- Test with LFO enabled at high amplitude to verify modulation works
- Check that mapped parameters actually affect the output

**Verification Checklist:**
- [ ] All Block 1 LFO panel parameters work
- [ ] All Block 2 LFO panel parameters work  
- [ ] All Block 3 LFO panel parameters work
- [ ] Component fields (x/y/z) sync with Vec3 fields
- [ ] Shear matrix components update correctly
- [ ] Colorize bands update correctly
- [ ] Matrix mixer components update correctly

**References:**
- `src/gui/mod.rs` - LFO panel definitions (search for `build_block1_lfo`)
- `src/core/lfo_engine.rs` - parameter application logic
- OF app macro system for complete parameter list

---

## Implementation Priority

| Priority | Feature | Effort | Impact | Notes |
|----------|---------|--------|--------|-------|
| **P0** | Text Scaling | Low | High | Blocks usability on high-DPI displays |
| **P0** | Shift+F Fullscreen | Low | High | Essential for performance |
| **P0** | Shift+T Tap Tempo | Low | Medium | Quick quality-of-life improvement |
| **P1** | Finalize LFO Mappings | Medium | High | Currently incomplete |
| **P1** | Pop-out Windows | High | High | Major workflow improvement |
| **P1** | MIDI Mapping | High | High | Professional control standard |
| **P2** | Layout Save/Load | Medium | Medium | Nice to have for power users |
| **P2** | OSC Addresses | Low | Low | Future-proofing for OSC control |
| **P2** | Framerate Control | Medium | Medium | Performance tuning |
| **P3** | Preview Improvements | High | Medium | Color picker is niche feature |

---

## Notes for Future Development

### Code Organization
- Keep GUI code in `src/gui/mod.rs`
- Keep engine/rendering code in `src/engine/`
- Keep parameter definitions in `src/params/mod.rs`
- Use `SharedState` for all communication between GUI and engine

### Configuration Files
- `config.toml` - Application settings
- `presets/` - Shader presets
- `layout.toml` - GUI layout (to be created)
- `midi_mappings.toml` - MIDI mappings (to be created)

### Performance Considerations
- Profile with `cargo flamegraph` for CPU bottlenecks
- Use `wgpu` profiling for GPU performance
- Target 60fps on M1 Mac, 30fps on older hardware
- Consider feature flags for optional heavy features

### Testing Strategy
- Test on macOS (primary platform)
- Test on Windows for MIDI compatibility
- Test with multiple monitors for layout features
- Test with various MIDI controllers (CC, notes, aftertouch)

---

## Related Documentation

- `AGENTS.md` - Technical reference for AI assistants
- `SIMPLE_ENGINE.md` - Simplified engine reference
- `ARCHITECTURE_PLAN.md` - Overall architecture
- `BLOCK1_IMPLEMENTATION_GUIDE.md` - Block 1 implementation details
- `LESSONS_LEARNED.md` - Development learnings and gotchas
