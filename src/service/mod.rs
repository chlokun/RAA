use anyhow::Result;
use std::path::PathBuf;

pub mod macos;
pub mod windows;

/// Common trait for background services across platforms
pub trait BackgroundService {
    fn install(&self) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn is_installed(&self) -> Result<bool>;
    fn is_running(&self) -> Result<bool>;
}

/// Get the platform-specific service implementation
pub fn get_service(name: &str, display_name: &str, description: &str, executable_path: PathBuf) -> Box<dyn BackgroundService> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsService::new(name, display_name, description, executable_path))
    }
    
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOsService::new(name, display_name, description, executable_path))
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Fallback for Linux or other platforms
        Box::new(linux::LinuxService::new(name, display_name, description, executable_path))
    }
}
