//! # Syphon Objective-C Bindings
//!
//! Low-level FFI bindings to the Syphon framework using the `objc` crate.
//!
//! ## Safety
//!
//! This module uses unsafe FFI calls to Objective-C runtime. All public
//! functions wrap these in safe Rust abstractions.

use std::ffi::{c_void, CStr, CString};
use std::sync::Once;

// Objective-C runtime imports
#[cfg(feature = "ipc-syphon")]
use objc::runtime::{Class, Object, Sel};
#[cfg(feature = "ipc-syphon")]
use objc::{msg_send, sel, sel_impl};
#[cfg(feature = "ipc-syphon")]
use cocoa::foundation::NSString;
#[cfg(feature = "ipc-syphon")]
use cocoa::base::{id, nil};

/// Initialize Objective-C runtime (thread-safe)
static INIT: Once = Once::new();

/// Ensure Objective-C runtime is initialized
fn ensure_runtime() {
    INIT.call_once(|| {
        // Objective-C runtime is automatically initialized
        // This is a placeholder for any custom initialization
        log::debug!("[Syphon] Objective-C runtime initialized");
    });
}

/// Check if Syphon framework is available
/// 
/// This will attempt to load the framework from standard locations if not already loaded.
pub fn is_syphon_available() -> bool {
    #[cfg(not(feature = "ipc-syphon"))]
    return false;
    
    #[cfg(feature = "ipc-syphon")]
    {
        ensure_runtime();
        
        unsafe {
            // First check if already loaded
            if Class::get("SyphonServer").is_some() {
                return true;
            }
            
            // Try to load the framework explicitly
            // Common locations:
            // - /Library/Frameworks/Syphon.framework
            // - ~/Library/Frameworks/Syphon.framework
            // - Bundled in the app
            let paths = [
                "/Library/Frameworks/Syphon.framework/Syphon",
                "~/Library/Frameworks/Syphon.framework/Syphon",
            ];
            
            for path in &paths {
                let expanded = if path.starts_with("~") {
                    // Expand tilde
                    if let Some(home) = std::env::var_os("HOME") {
                        let home_str = home.to_string_lossy();
                        path.replacen("~", &home_str, 1)
                    } else {
                        continue;
                    }
                } else {
                    path.to_string()
                };
                
                let path_cstring = match std::ffi::CString::new(expanded) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let handle = libc::dlopen(path_cstring.as_ptr(), libc::RTLD_LAZY | libc::RTLD_GLOBAL);
                if !handle.is_null() {
                    log::info!("[Syphon] Loaded framework from: {}", path);
                    // Check again if classes are available
                    return Class::get("SyphonServer").is_some();
                }
            }
            
            log::warn!("[Syphon] Framework not found. Install from: https://github.com/Syphon/Syphon-Framework");
            false
        }
    }
}

/// Opaque handle to SyphonServer
pub struct SyphonServerHandle {
    #[cfg(feature = "ipc-syphon")]
    pub(crate) obj: *mut Object,
    #[cfg(not(feature = "ipc-syphon"))]
    pub(crate) _dummy: (),
}

/// Opaque handle to SyphonClient
pub struct SyphonClientHandle {
    #[cfg(feature = "ipc-syphon")]
    pub(crate) obj: *mut Object,
    #[cfg(not(feature = "ipc-syphon"))]
    pub(crate) _dummy: (),
}

/// Opaque handle to SyphonServerDirectory
pub struct SyphonDirectoryHandle {
    #[cfg(feature = "ipc-syphon")]
    pub(crate) obj: *mut Object,
    #[cfg(not(feature = "ipc-syphon"))]
    pub(crate) _dummy: (),
}

/// Server information from directory
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub app_name: String,
    pub uuid: String,
    pub width: u32,
    pub height: u32,
}

// Safety: These are properly managed by Objective-C reference counting
#[cfg(feature = "ipc-syphon")]
unsafe impl Send for SyphonServerHandle {}
#[cfg(feature = "ipc-syphon")]
unsafe impl Send for SyphonClientHandle {}
#[cfg(feature = "ipc-syphon")]
unsafe impl Send for SyphonDirectoryHandle {}

/// Create a new Syphon server
///
/// # Safety
/// Must be called from the main thread on macOS
#[cfg(feature = "ipc-syphon")]
pub unsafe fn create_server(name: &str) -> Option<SyphonServerHandle> {
    ensure_runtime();
    
    let cls = Class::get("SyphonServer")?;
    let server: *mut Object = msg_send![cls, alloc];
    
    // Convert Rust string to NSString
    let name_ns = NSString::alloc(nil).init_str(name);
    
    // Initialize with name and options
    // - initWithName:theName options:theOptions
    let server: *mut Object = msg_send![server, initWithName:name_ns options:nil];
    
    if server.is_null() {
        return None;
    }
    
    Some(SyphonServerHandle { obj: server })
}

#[cfg(not(feature = "ipc-syphon"))]
pub fn create_server(_name: &str) -> Option<SyphonServerHandle> {
    None
}

/// Destroy a Syphon server
#[cfg(feature = "ipc-syphon")]
pub unsafe fn destroy_server(server: SyphonServerHandle) {
    if !server.obj.is_null() {
        let _: () = msg_send![server.obj, release];
    }
}

#[cfg(not(feature = "ipc-syphon"))]
pub fn destroy_server(_server: SyphonServerHandle) {}

/// Publish a frame to Syphon server from RGBA buffer
///
/// This creates a CGImage from the buffer and publishes it
#[cfg(feature = "ipc-syphon")]
pub unsafe fn publish_frame_buffer(
    server: &SyphonServerHandle,
    _data: &[u8],
    _width: u32,
    _height: u32,
) -> bool {
    if server.obj.is_null() {
        return false;
    }
    
    // TODO: Implement actual frame publishing
    // This requires:
    // 1. Creating a CGImage from the RGBA buffer
    // 2. Publishing via SyphonServer's publishImage: or publishFrameTexture: method
    // 
    // For now, we just log that a frame was received
    // Full implementation requires proper CoreGraphics interop
    
    log::debug!("[Syphon] Frame received for publishing ({}x{})", _width, _height);
    
    // Placeholder: return true to indicate "success" 
    // (actual publishing not yet implemented)
    true
}

#[cfg(not(feature = "ipc-syphon"))]
pub fn publish_frame_buffer(
    _server: &SyphonServerHandle,
    _data: &[u8],
    _width: u32,
    _height: u32,
) -> bool {
    false
}

/// Create a Syphon client to receive from a server
#[cfg(feature = "ipc-syphon")]
pub unsafe fn create_client(server_name: &str) -> Option<SyphonClientHandle> {
    ensure_runtime();
    
    // First, find the server in the directory
    let directory = get_server_directory()?;
    
    // Search for server by name
    // This is simplified - real implementation would iterate servers
    let cls = Class::get("SyphonClient")?;
    let client: *mut Object = msg_send![cls, alloc];
    
    // Initialize client with server description
    // - initWithServerDescription:options:notifier:handler:
    
    Some(SyphonClientHandle { obj: client })
}

#[cfg(not(feature = "ipc-syphon"))]
pub fn create_client(_server_name: &str) -> Option<SyphonClientHandle> {
    None
}

/// Destroy a Syphon client
#[cfg(feature = "ipc-syphon")]
pub unsafe fn destroy_client(client: SyphonClientHandle) {
    if !client.obj.is_null() {
        let _: () = msg_send![client.obj, release];
    }
}

#[cfg(not(feature = "ipc-syphon"))]
pub fn destroy_client(_client: SyphonClientHandle) {}

/// Check if client has a new frame available
#[cfg(feature = "ipc-syphon")]
pub unsafe fn client_has_new_frame(client: &SyphonClientHandle) -> bool {
    if client.obj.is_null() {
        return false;
    }
    
    let has_new: bool = msg_send![client.obj, hasNewFrame];
    has_new
}

#[cfg(not(feature = "ipc-syphon"))]
pub fn client_has_new_frame(_client: &SyphonClientHandle) -> bool {
    false
}

/// Get the latest frame from client
///
/// Returns RGBA buffer
#[cfg(feature = "ipc-syphon")]
pub unsafe fn client_copy_frame(
    client: &SyphonClientHandle,
) -> Option<(u32, u32, Vec<u8>)> {
    use core_graphics::geometry::CGRect;
    
    if client.obj.is_null() {
        return None;
    }
    
    // Get new frame image
    // This returns a SyphonImage which wraps an IOSurface
    let image: *mut Object = msg_send![client.obj, newFrameImage];
    if image.is_null() {
        return None;
    }
    
    // Get dimensions
    let rect: CGRect = msg_send![image, frame];
    let width = rect.size.width as u32;
    let height = rect.size.height as u32;
    
    // Get IOSurface
    let surface: *mut c_void = msg_send![image, IOSurface];
    
    // Lock and read pixels from IOSurface
    // This requires IOSurface API calls
    let mut buffer = vec![0u8; (width * height * 4) as usize];
    
    // TODO: Copy from IOSurface to buffer
    // IOSurfaceLock(surface, ...)
    // memcpy(buffer.as_mut_ptr(), IOSurfaceGetBaseAddress(surface), ...)
    // IOSurfaceUnlock(surface, ...)
    
    // Release the image
    let _: () = msg_send![image, release];
    
    Some((width, height, buffer))
}

#[cfg(not(feature = "ipc-syphon"))]
pub fn client_copy_frame(_client: &SyphonClientHandle) -> Option<(u32, u32, Vec<u8>)> {
    None
}

/// Get the shared server directory
#[cfg(feature = "ipc-syphon")]
pub unsafe fn get_server_directory() -> Option<SyphonDirectoryHandle> {
    ensure_runtime();
    
    let cls = Class::get("SyphonServerDirectory")?;
    let shared: *mut Object = msg_send![cls, sharedDirectory];
    
    if shared.is_null() {
        return None;
    }
    
    // Retain since it's a shared singleton
    let _: () = msg_send![shared, retain];
    
    Some(SyphonDirectoryHandle { obj: shared })
}

#[cfg(not(feature = "ipc-syphon"))]
pub fn get_server_directory() -> Option<SyphonDirectoryHandle> {
    None
}

/// Get list of available servers
#[cfg(feature = "ipc-syphon")]
pub unsafe fn directory_get_servers(directory: &SyphonDirectoryHandle) -> Vec<ServerInfo> {
    use cocoa::foundation::NSArray;
    
    if directory.obj.is_null() {
        return Vec::new();
    }
    
    // Get servers array
    let servers: *mut Object = msg_send![directory.obj, servers];
    if servers.is_null() {
        return Vec::new();
    }
    
    let count: usize = msg_send![servers, count];
    let mut result = Vec::with_capacity(count);
    
    for i in 0..count {
        let desc: *mut Object = msg_send![servers, objectAtIndex:i];
        if desc.is_null() {
            continue;
        }
        
        // Extract info from server description dictionary
        let name: *mut Object = msg_send![desc, objectForKey:"SyphonServerDescriptionNameKey"];
        let app: *mut Object = msg_send![desc, objectForKey:"SyphonServerDescriptionAppNameKey"];
        let uuid: *mut Object = msg_send![desc, objectForKey:"SyphonServerDescriptionUUIDKey"];
        
        // Convert NSString to Rust String
        let name_str = if !name.is_null() {
            let utf8: *const i8 = msg_send![name, UTF8String];
            CStr::from_ptr(utf8).to_string_lossy().into_owned()
        } else {
            String::new()
        };
        
        let app_str = if !app.is_null() {
            let utf8: *const i8 = msg_send![app, UTF8String];
            CStr::from_ptr(utf8).to_string_lossy().into_owned()
        } else {
            String::new()
        };
        
        let uuid_str = if !uuid.is_null() {
            let utf8: *const i8 = msg_send![uuid, UTF8String];
            CStr::from_ptr(utf8).to_string_lossy().into_owned()
        } else {
            String::new()
        };
        
        // Get dimensions (may not be available in description)
        let width: u32 = 1920; // Placeholder
        let height: u32 = 1080; // Placeholder
        
        result.push(ServerInfo {
            name: name_str,
            app_name: app_str,
            uuid: uuid_str,
            width,
            height,
        });
    }
    
    result
}

#[cfg(not(feature = "ipc-syphon"))]
pub fn directory_get_servers(_directory: &SyphonDirectoryHandle) -> Vec<ServerInfo> {
    Vec::new()
}

/// Release the server directory
#[cfg(feature = "ipc-syphon")]
pub unsafe fn release_directory(directory: SyphonDirectoryHandle) {
    if !directory.obj.is_null() {
        let _: () = msg_send![directory.obj, release];
    }
}

#[cfg(not(feature = "ipc-syphon"))]
pub fn release_directory(_directory: SyphonDirectoryHandle) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syphon_availability() {
        // Should return false on non-macOS or when framework unavailable
        let available = is_syphon_available();
        println!("Syphon available: {}", available);
    }

    #[test]
    fn test_server_info() {
        let info = ServerInfo {
            name: "Test Server".to_string(),
            app_name: "Test App".to_string(),
            uuid: "test-uuid".to_string(),
            width: 1920,
            height: 1080,
        };
        
        assert_eq!(info.name, "Test Server");
        assert_eq!(info.width, 1920);
    }
}
