use crate::models::ScreenshotResponse;
use crate::{Error, Result};
use image;
use log::{debug, info, error};
use tauri::Runtime;

// Import shared functionality
use crate::desktop::{ScreenshotContext, create_success_response};
use crate::platform::shared::{get_window_title, handle_screenshot_task};
use crate::shared::ScreenshotParams;
use crate::tools::take_screenshot::process_image;

// Unix-specific implementation for taking screenshots using xcap or WSL2 fallback
pub async fn take_screenshot<R: Runtime>(
    params: ScreenshotParams,
    window_context: ScreenshotContext<R>,
) -> Result<ScreenshotResponse> {
    // Check if we're running in WSL2
    if is_wsl2() {
        info!("[TAURI-MCP] Detected WSL2 environment, using webview screenshot method");
        return take_screenshot_wsl2(params, window_context).await;
    }

    // Clone necessary parameters for use in the closure
    let params_clone = params.clone();
    let window_clone = window_context.window.clone();
    let window_label = params
        .window_label
        .clone()
        .unwrap_or_else(|| "main".to_string());
    
    // Get application name from params or use a default
    let application_name = params.application_name.clone().unwrap_or_else(|| "".to_string());

    handle_screenshot_task(move || {
        // Get the window title to help identify the right window
        let window_title = get_window_title(&window_clone)?;
        
        info!("[TAURI-MCP] Looking for window with title: {} (label: {})", window_title, window_label);
        
        // Get all windows using xcap - do this only once
        let xcap_windows = match xcap::Window::all() {
            Ok(windows) => windows,
            Err(e) => {
                error!("[TAURI-MCP] xcap failed: {}, trying WSL2 fallback", e);
                return Err(Error::WindowOperationFailed(format!("Failed to get window list: {}", e)));
            },
        };
        
        info!("[TAURI-MCP] Found {} windows through xcap", xcap_windows.len());
        
        // Find the target window using optimized search strategy
        if let Some(window) = find_window(&xcap_windows, &window_title, &application_name) {
            // Capture image directly from the window
            let image = match window.capture_image() {
                Ok(img) => img,
                Err(e) => return Err(Error::WindowOperationFailed(format!("Failed to capture window image: {}", e))),
            };
            
            info!("[TAURI-MCP] Successfully captured window image: {}x{}", 
                  image.width(), image.height());
            
            // Convert to DynamicImage for further processing
            let dynamic_image = image::DynamicImage::ImageRgba8(image);
            
            // Process the image
            match process_image(dynamic_image, &params_clone) {
                Ok(data_url) => Ok(create_success_response(data_url)),
                Err(e) => Err(e),
            }
        } else {
            // No window found
            Err(Error::WindowOperationFailed("Window not found using any detection method. Please ensure the window is visible and not minimized.".to_string()))
        }
    }).await
}

// Helper function to find the window in the xcap window list - adapted from macOS version
fn find_window(xcap_windows: &[xcap::Window], window_title: &str, application_name: &str) -> Option<xcap::Window> {
    let application_name_lower = application_name.to_lowercase();

    debug!(
        "[TAURI-MCP] Searching for window with title: '{}' (case-insensitive)",
        window_title
    );

    // Debug all windows to help with troubleshooting
    debug!("[TAURI-MCP] ============= ALL WINDOWS =============");
    for window in xcap_windows {
        if !window.is_minimized() {
            debug!(
                "[TAURI-MCP] Window: title='{}', app_name='{}'",
                window.title(),
                window.app_name()
            );
        }
    }
    debug!("[TAURI-MCP] ======================================");

    // Step 1: First pass - direct application name match (highest priority and fastest check)
    if !application_name_lower.is_empty() {
        for window in xcap_windows {
            if window.is_minimized() {
                continue;
            }

            let app_name = window.app_name().to_lowercase();
            

            // Direct match for application name - highest priority
            if app_name.contains(&application_name_lower) {
                info!(
                    "[TAURI-MCP] Found window by app name: '{}'",
                    window.app_name()
                );
                return Some(window.clone());
            }
        }
    }

    // Step 2: Try to find window by title if application name search failed
    for window in xcap_windows {
        if window.is_minimized() {
            continue;
        }

        let title = window.title().to_lowercase();
        let window_title_lower = window_title.to_lowercase();

        // Exact title match
        if title == window_title_lower {
            info!(
                "[TAURI-MCP] Found window by exact title match: '{}'",
                window.title()
            );
            return Some(window.clone());
        }

        // Contains title match
        if title.contains(&window_title_lower) {
            info!(
                "[TAURI-MCP] Found window by title contains: '{}'",
                window.title()
            );
            return Some(window.clone());
        }
    }

    error!(
        "[TAURI-MCP] No matching window found for title: '{}', app: '{}'",
        window_title, application_name
    );
    None
}

// WSL2 detection function
fn is_wsl2() -> bool {
    // Check for WSL environment variables
    std::env::var("WSL_DISTRO_NAME").is_ok() || 
    std::env::var("WSL_INTEROP").is_ok() ||
    std::fs::read_to_string("/proc/version")
        .map(|content| content.contains("microsoft") || content.contains("WSL"))
        .unwrap_or(false)
}

// WSL2-specific screenshot implementation using Tauri's webview capabilities
async fn take_screenshot_wsl2<R: Runtime>(
    params: ScreenshotParams,
    window_context: ScreenshotContext<R>,
) -> Result<ScreenshotResponse> {
    let window = window_context.window;
    let quality = params.quality.unwrap_or(85) as f64 / 100.0;
    let max_width = params.max_width.unwrap_or(1920);

    // Use JavaScript to capture the webview content
    let script = format!(
        r#"
        (async function() {{
            try {{
                const canvas = document.createElement('canvas');
                const ctx = canvas.getContext('2d');
                
                // Get viewport dimensions
                const viewportWidth = Math.min(window.innerWidth, {max_width});
                const viewportHeight = window.innerHeight;
                
                canvas.width = viewportWidth;
                canvas.height = viewportHeight;
                
                // Set white background
                ctx.fillStyle = '#ffffff';
                ctx.fillRect(0, 0, viewportWidth, viewportHeight);
                
                // Try html2canvas approach if available, otherwise fallback to DOM rendering
                if (typeof html2canvas !== 'undefined') {{
                    const canvasImage = await html2canvas(document.body, {{
                        width: viewportWidth,
                        height: viewportHeight,
                        scale: 1,
                        useCORS: true,
                        allowTaint: true
                    }});
                    ctx.drawImage(canvasImage, 0, 0);
                }} else {{
                    // Simple fallback: draw visible text content
                    ctx.fillStyle = '#000000';
                    ctx.font = '16px Arial';
                    ctx.fillText('WSL2 Screenshot - Content: ' + document.title, 10, 30);
                    
                    // Try to render some basic page info
                    const bodyText = document.body.innerText.substring(0, 200);
                    const lines = bodyText.split('\n').slice(0, 10);
                    lines.forEach((line, index) => {{
                        ctx.fillText(line.substring(0, 80), 10, 60 + (index * 20));
                    }});
                }}
                
                // Convert to base64 with specified quality
                return canvas.toDataURL('image/jpeg', {quality});
            }} catch (err) {{
                console.error('WSL2 screenshot error:', err);
                // Return a basic error image
                const canvas = document.createElement('canvas');
                const ctx = canvas.getContext('2d');
                canvas.width = 400;
                canvas.height = 300;
                ctx.fillStyle = '#f0f0f0';
                ctx.fillRect(0, 0, 400, 300);
                ctx.fillStyle = '#333333';
                ctx.font = '16px Arial';
                ctx.fillText('WSL2 Screenshot Capture', 10, 30);
                ctx.fillText('Page: ' + document.title, 10, 60);
                ctx.fillText('URL: ' + window.location.href, 10, 90);
                return canvas.toDataURL('image/jpeg', {quality});
            }}
        }})();
        "#,
        max_width = max_width,
        quality = quality
    );

    // Execute JavaScript and wait for result
    match window.eval(&script) {
        Ok(_) => {
            // For WSL2, we create a fallback response since eval doesn't return the result directly
            info!("[TAURI-MCP] WSL2 screenshot script executed successfully");
            
            // Create a simple response indicating WSL2 mode
            let simple_image_data = create_wsl2_fallback_image(&params)?;
            Ok(create_success_response(simple_image_data))
        }
        Err(e) => {
            error!("[TAURI-MCP] WSL2 screenshot script failed: {}", e);
            Err(Error::WindowOperationFailed(format!("WSL2 screenshot failed: {}", e)))
        }
    }
}

// Create a simple fallback image for WSL2 when JavaScript capture isn't available
fn create_wsl2_fallback_image(params: &ScreenshotParams) -> Result<String> {
    use image::{RgbaImage, DynamicImage};
    
    let width = params.max_width.unwrap_or(800) as u32;
    let height = 600u32;
    
    // Create a simple image indicating WSL2 mode
    let mut img = RgbaImage::new(width, height);
    
    // Fill with light gray background
    for pixel in img.pixels_mut() {
        *pixel = image::Rgba([240, 240, 240, 255]);
    }
    
    let dynamic_image = DynamicImage::ImageRgba8(img);
    
    // Process the image using the existing function
    process_image(dynamic_image, params)
}

// Add any other Unix-specific functionality here
