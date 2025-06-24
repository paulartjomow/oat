use seahorse::{Command, Context, Flag, FlagType};
use std::sync::mpsc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    published_at: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug)]
pub enum UpdateError {
    NetworkError(String),
    ParseError(String),
    UpdateError(String),
    NoUpdateNeeded,
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            UpdateError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            UpdateError::UpdateError(msg) => write!(f, "Update error: {}", msg),
            UpdateError::NoUpdateNeeded => write!(f, "No update needed"),
        }
    }
}

impl Error for UpdateError {}

pub fn update_command() -> Command {
    Command::new("update")
        .description("Check for updates and update the application")
        .usage("oat update [--check-only]")
        .flag(
            Flag::new("check-only", FlagType::Bool)
                .description("Only check for updates, don't install")
                .alias("c"),
        )
        .action(update_action)
}

fn update_action(c: &Context) {
    let check_only = c.bool_flag("check-only");
    
    // Create a new thread to avoid the nested runtime issue
    let (tx, rx) = mpsc::channel();
    
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            if check_only {
                check_for_updates().await
            } else {
                perform_update().await
            }
        });
        tx.send(result).unwrap();
    });
    
    match rx.recv().unwrap() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Update failed: {}", e);
            std::process::exit(1);
        }
    }
}

async fn check_for_updates() -> Result<(), UpdateError> {
    println!("🔍 Checking for updates...");
    
    let current_version = env!("CARGO_PKG_VERSION");
    let latest_release = get_latest_release().await?;
    
    let latest_version = latest_release.tag_name.trim_start_matches('v');
    
    println!("Current version: v{}", current_version);
    println!("Latest version: {}", latest_release.tag_name);
    
    match compare_versions(current_version, latest_version)? {
        std::cmp::Ordering::Less => {
            println!("✅ New version available!");
            println!("Release notes:");
            println!("{}", latest_release.body);
            println!("\nRun 'oat update' to install the latest version.");
        }
        std::cmp::Ordering::Equal => {
            println!("✅ You're running the latest version!");
        }
        std::cmp::Ordering::Greater => {
            println!("🚀 You're running a newer version than the latest release!");
        }
    }
    
    Ok(())
}

async fn perform_update() -> Result<(), UpdateError> {
    println!("🔍 Checking for updates...");
    
    let current_version = env!("CARGO_PKG_VERSION");
    let latest_release = get_latest_release().await?;
    
    let latest_version = latest_release.tag_name.trim_start_matches('v');
    
    match compare_versions(current_version, latest_version)? {
        std::cmp::Ordering::Less => {
            println!("📦 New version {} available!", latest_release.tag_name);
            println!("Current version: v{}", current_version);
            
            // Ask for user confirmation
            println!("\nRelease notes:");
            println!("{}", latest_release.body);
            println!("\nDo you want to update? (y/N)");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).map_err(|e| {
                UpdateError::UpdateError(format!("Failed to read input: {}", e))
            })?;
            
            if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
                println!("Update cancelled.");
                return Ok(());
            }
            
            install_update().await?;
        }
        std::cmp::Ordering::Equal => {
            println!("✅ You're already running the latest version (v{})!", current_version);
            return Err(UpdateError::NoUpdateNeeded);
        }
        std::cmp::Ordering::Greater => {
            println!("🚀 You're running a newer version (v{}) than the latest release!", current_version);
            return Err(UpdateError::NoUpdateNeeded);
        }
    }
    
    Ok(())
}

async fn get_latest_release() -> Result<GitHubRelease, UpdateError> {
    let client = reqwest::Client::new();
    let url = "https://api.github.com/repos/Prixix/oat/releases/latest";
    
    let response = client
        .get(url)
        .header("User-Agent", "oat-cli")
        .send()
        .await
        .map_err(|e| UpdateError::NetworkError(format!("Failed to fetch release info: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(UpdateError::NetworkError(format!(
            "GitHub API returned status: {}",
            response.status()
        )));
    }
    
    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| UpdateError::ParseError(format!("Failed to parse release info: {}", e)))?;
    
    Ok(release)
}

fn compare_versions(current: &str, latest: &str) -> Result<std::cmp::Ordering, UpdateError> {
    let current_version = semver::Version::parse(current)
        .map_err(|e| UpdateError::ParseError(format!("Invalid current version: {}", e)))?;
    
    let latest_version = semver::Version::parse(latest)
        .map_err(|e| UpdateError::ParseError(format!("Invalid latest version: {}", e)))?;
    
    Ok(current_version.cmp(&latest_version))
}

async fn install_update() -> Result<(), UpdateError> {
    println!("🚀 Installing update...");
    
    let target = get_target_triple();
    let bin_name = env!("CARGO_PKG_NAME");
    
    let status = self_update::backends::github::Update::configure()
        .repo_owner("Prixix")
        .repo_name("oat")
        .bin_name(bin_name)
        .target(&target)
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()
        .map_err(|e| UpdateError::UpdateError(format!("Failed to configure updater: {}", e)))?
        .update()
        .map_err(|e| UpdateError::UpdateError(format!("Failed to update: {}", e)))?;
    
    match status {
        self_update::Status::UpToDate(version) => {
            println!("✅ Already up to date (version {})!", version);
        }
        self_update::Status::Updated(version) => {
            println!("✅ Successfully updated to version {}!", version);
            println!("🎉 Restart the application to use the new version.");
        }
    }
    
    Ok(())
}

fn get_target_triple() -> String {
    // Determine the target triple based on the current platform
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "x86_64-apple-darwin".to_string();
    
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "aarch64-apple-darwin".to_string();
    
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "x86_64-unknown-linux-gnu".to_string();
    
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "aarch64-unknown-linux-gnu".to_string();
    
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "x86_64-pc-windows-msvc".to_string();
    
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return "aarch64-pc-windows-msvc".to_string();
    
    // Fallback for unsupported platforms
    #[cfg(not(any(
        all(target_os = "macos", any(target_arch = "x86_64", target_arch = "aarch64")),
        all(target_os = "linux", any(target_arch = "x86_64", target_arch = "aarch64")),
        all(target_os = "windows", any(target_arch = "x86_64", target_arch = "aarch64"))
    )))]
    return "x86_64-unknown-linux-gnu".to_string();
}

// Auto-update check function that can be called on startup
pub async fn check_auto_update() -> Result<(), UpdateError> {
    // Check if auto-update check is enabled (you can add a config file later)
    let should_check = std::env::var("OAT_AUTO_UPDATE_CHECK").unwrap_or_else(|_| "true".to_string());
    
    if should_check.to_lowercase() != "true" {
        return Ok(());
    }
    
    // Check if we should perform an auto-update check (e.g., once per day)
    if should_perform_auto_check() {
        println!("🔍 Checking for updates in the background...");
        
        match get_latest_release().await {
            Ok(latest_release) => {
                let current_version = env!("CARGO_PKG_VERSION");
                let latest_version = latest_release.tag_name.trim_start_matches('v');
                
                if let Ok(std::cmp::Ordering::Less) = compare_versions(current_version, latest_version) {
                    println!("💡 New version {} is available! Run 'oat update' to upgrade.", latest_release.tag_name);
                }
            }
            Err(_) => {
                // Silently fail for auto-checks to avoid annoying users
            }
        }
    }
    
    Ok(())
}

fn should_perform_auto_check() -> bool {
    // Simple implementation - check if last check was more than 24 hours ago
    // You can enhance this by storing the last check time in a config file
    
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let last_check_file = home_dir.join(".oat_last_update_check");
    
    if !last_check_file.exists() {
        // First time, create the file and return true
        let _ = std::fs::write(&last_check_file, chrono::Utc::now().timestamp().to_string());
        return true;
    }
    
    if let Ok(content) = std::fs::read_to_string(&last_check_file) {
        if let Ok(last_check) = content.trim().parse::<i64>() {
            let now = chrono::Utc::now().timestamp();
            let hours_since_check = (now - last_check) / 3600;
            
            if hours_since_check >= 24 {
                // Update the last check time
                let _ = std::fs::write(&last_check_file, now.to_string());
                return true;
            }
        }
    }
    
    false
} 