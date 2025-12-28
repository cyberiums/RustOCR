use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// Get the path to the server script
pub fn get_server_script_path() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()
        .context("Failed to get executable path")?;
    let exe_dir = exe_path.parent()
        .context("Failed to get executable directory")?;
    
    let server_script = exe_dir.join("easyocr_server.py");
    
    if !server_script.exists() {
        let cwd_server = PathBuf::from("easyocr_server.py");
        if cwd_server.exists() {
            return Ok(cwd_server);
        }
    }
    
    Ok(server_script)
}

/// Start the server in background
pub fn start_server(port: u16, host: &str) -> Result<Child> {
    let server_script = get_server_script_path()?;
    
    if !server_script.exists() {
        anyhow::bail!(
            "Server script not found at: {}. Make sure easyocr_server.py is available.",
            server_script.display()
        );
    }
    
    eprintln!("Starting EasyOCR server on {}:{}...", host, port);
    eprintln!("Server script: {}", server_script.display());
    
    let child = Command::new("python3")
        .arg(&server_script)
        .arg("--host")
        .arg(host)
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start server process")?;
    
    // Wait a moment for server to start
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    Ok(child)
}

/// Save server PID to file
pub fn save_server_pid(pid: u32) -> Result<()> {
    let pid_file = get_pid_file_path();
    fs::write(&pid_file, pid.to_string())
        .context("Failed to write PID file")?;
    eprintln!("Server PID saved to: {}", pid_file.display());
    Ok(())
}

/// Get PID file path
fn get_pid_file_path() -> PathBuf {
    PathBuf::from("/tmp/rustocr_server.pid")
}

/// Read server PID from file
pub fn read_server_pid() -> Option<u32> {
    let pid_file = get_pid_file_path();
    fs::read_to_string(&pid_file)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// Stop the server
pub fn stop_server() -> Result<()> {
    if let Some(pid) = read_server_pid() {
        eprintln!("Stopping server with PID {}...", pid);
        
        // Try to kill the process
        #[cfg(unix)]
        {
            use std::process::Command;
            let _ = Command::new("kill")
                .arg(pid.to_string())
                .output();
        }
        
        // Remove PID file
        let pid_file = get_pid_file_path();
        let _ = fs::remove_file(&pid_file);
        
        eprintln!("Server stopped.");
        Ok(())
    } else {
        eprintln!("No running server found.");
        Ok(())
    }
}

/// Check if server is running
pub fn is_server_running() -> bool {
    if let Some(pid) = read_server_pid() {
        // Check if process exists
        #[cfg(unix)]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("ps")
                .arg("-p")
                .arg(pid.to_string())
                .output()
            {
                return output.status.success();
            }
        }
    }
    false
}
