use anyhow::{Result, Context};
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use super::BackgroundService;

pub struct MacOsService {
    name: String,
    display_name: String,
    description: String,
    executable_path: PathBuf,
}

impl MacOsService {
    pub fn new(name: &str, display_name: &str, description: &str, executable_path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            description: description.to_string(),
            executable_path,
        }
    }
    
    fn get_plist_path(&self) -> PathBuf {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        home_dir.join("Library/LaunchAgents").join(format!("com.{}.plist", self.name))
    }
    
    fn create_plist_file(&self) -> Result<()> {
        let plist_path = self.get_plist_path();
        
        // Ensure the LaunchAgents directory exists
        if let Some(parent) = plist_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create LaunchAgents directory")?;
        }
        
        let executable = self.executable_path.to_str()
            .context("Invalid executable path")?;
            
        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>{}/Library/Logs/{}.log</string>
    <key>StandardOutPath</key>
    <string>{}/Library/Logs/{}.log</string>
</dict>
</plist>"#,
            self.name,
            executable,
            home_dir_str(),
            self.name,
            home_dir_str(),
            self.name
        );
        
        let mut file = File::create(&plist_path)
            .context("Failed to create plist file")?;
            
        file.write_all(plist_content.as_bytes())
            .context("Failed to write plist content")?;
            
        Ok(())
    }
}

impl BackgroundService for MacOsService {
    fn install(&self) -> Result<()> {
        // Create the plist file
        self.create_plist_file()?;
        
        // Register with launchctl
        let status = std::process::Command::new("launchctl")
            .args(&["load", "-w"])
            .arg(self.get_plist_path())
            .status()
            .context("Failed to execute launchctl")?;
            
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to load service with launchctl"));
        }
        
        log::info!("Installed macOS service: {}", self.name);
        Ok(())
    }
    
    fn uninstall(&self) -> Result<()> {
        // Unregister with launchctl
        let plist_path = self.get_plist_path();
        
        if plist_path.exists() {
            let status = std::process::Command::new("launchctl")
                .args(&["unload", "-w"])
                .arg(&plist_path)
                .status()
                .context("Failed to execute launchctl")?;
                
            if !status.success() {
                log::warn!("Failed to unload service with launchctl");
            }
            
            // Remove the plist file
            std::fs::remove_file(&plist_path)
                .context("Failed to remove plist file")?;
        }
        
        log::info!("Uninstalled macOS service: {}", self.name);
        Ok(())
    }
    
    fn start(&self) -> Result<()> {
        let status = std::process::Command::new("launchctl")
            .args(&["start", &format!("com.{}", self.name)])
            .status()
            .context("Failed to execute launchctl")?;
            
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to start service with launchctl"));
        }
        
        log::info!("Started macOS service: {}", self.name);
        Ok(())
    }
    
    fn stop(&self) -> Result<()> {
        let status = std::process::Command::new("launchctl")
            .args(&["stop", &format!("com.{}", self.name)])
            .status()
            .context("Failed to execute launchctl")?;
            
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to stop service with launchctl"));
        }
        
        log::info!("Stopped macOS service: {}", self.name);
        Ok(())
    }
    
    fn is_installed(&self) -> Result<bool> {
        Ok(self.get_plist_path().exists())
    }
    
    fn is_running(&self) -> Result<bool> {
        let output = std::process::Command::new("launchctl")
            .args(&["list"])
            .output()
            .context("Failed to execute launchctl")?;
            
        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.contains(&format!("com.{}", self.name)))
    }
}

fn home_dir_str() -> String {
    dirs::home_dir()
        .expect("Could not find home directory")
        .to_str()
        .expect("Home directory path is not valid UTF-8")
        .to_string()
}
