use anyhow::{Context, Result};
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};
use crossbeam_channel::{bounded, select, Receiver};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Watch mode for automatic OCR processing of new files
/// 
/// Monitors a directory and automatically processes new image files

pub struct WatchConfig {
    pub watch_dir: PathBuf,
    pub output_dir: Option<PathBuf>,
    pub archive_dir: Option<PathBuf>,
    pub recursive: bool,
    pub extensions: Vec<String>,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            watch_dir: PathBuf::from("."),
            output_dir: None,
            archive_dir: None,
            recursive: false,
            extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "bmp".to_string(),
                "tiff".to_string(),
            ],
        }
    }
}

/// Start watching directory for new files
pub fn watch_directory<F>(config: WatchConfig, processor: F) -> Result<()>
where
    F: Fn(&Path) -> Result<()> + Send + 'static,
{
    let (tx, rx) = bounded(100);
    
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        notify::Config::default(),
    )?;

    let mode = if config.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    watcher.watch(&config.watch_dir, mode)
        .context("Failed to start watching directory")?;

    eprintln!("👁️  Watching directory: {}", config.watch_dir.display());
    eprintln!("   Extensions: {}", config.extensions.join(", "));
    eprintln!("   Press Ctrl+C to stop");

    let processor = std::sync::Arc::new(processor);
    
    loop {
        select! {
            recv(rx) -> event => {
                if let Ok(event) = event {
                    handle_event(event, &config, &processor)?;
                }
            }
        }
    }
}

fn handle_event<F>(event: Event, config: &WatchConfig, processor: &F) -> Result<()>
where
    F: Fn(&Path) -> Result<()>,
{
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {
            for path in event.paths {
                if should_process(&path, config) {
                    eprintln!("📄 New file detected: {}", path.display());
                    
                    // Small delay to ensure file is fully written
                    std::thread::sleep(Duration::from_millis(500));
                    
                    match processor(&path) {
                        Ok(_) => {
                            eprintln!("✅ Processed: {}", path.display());
                            
                            // Move to archive if configured
                            if let Some(archive_dir) = &config.archive_dir {
                                archive_file(&path, archive_dir)?;
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Error processing {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
        _ => {}
    }
    
    Ok(())
}

fn should_process(path: &Path, config: &WatchConfig) -> bool {
    if !path.is_file() {
        return false;
    }
    
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        config.extensions.iter().any(|e| e == &ext_str)
    } else {
        false
    }
}

fn archive_file(path: &Path, archive_dir: &Path) -> Result<()> {
    use std::fs;
    
    fs::create_dir_all(archive_dir)?;
    
    if let Some(filename) = path.file_name() {
        let archive_path = archive_dir.join(filename);
        fs::rename(path, &archive_path)
            .context("Failed to move file to archive")?;
        eprintln!("📦 Archived to: {}", archive_path.display());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();
        assert!(!config.recursive);
        assert!(config.extensions.contains(&"jpg".to_string()));
    }

    #[test]
    fn test_should_process() {
        let config = WatchConfig::default();
        let path = PathBuf::from("test.jpg");
        // Would need actual file to test fully
        assert!(!should_process(&path, &config)); // File doesn't exist
    }
}
