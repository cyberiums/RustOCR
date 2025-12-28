use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod client;
mod server;
use client::{check_server_health, ocr_via_server, OcrResult};

/// RustOCR - A fast Rust CLI for EasyOCR with 80+ language support
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input image file path
    #[arg(short, long)]
    input: Option<String>,

    /// Languages to recognize (comma-separated, e.g., "en" or "ch_sim,en")
    #[arg(short, long, default_value = "en", value_delimiter = ',')]
    languages: Vec<String>,

    /// Enable GPU acceleration (requires CUDA)
    #[arg(short, long, default_value = "true")]
    gpu: bool,

    /// Detail level: 0 for text only, 1 for bounding boxes and confidence
    #[arg(short, long, default_value = "1")]
    detail: i32,

    /// Output format: json, text, or detailed
    #[arg(short, long, default_value = "json")]
    output: String,
    
    /// Use server mode instead of subprocess (faster for multiple requests)
    #[arg(long)]
    use_server: bool,
    
    /// Server URL (default: http://localhost:8000)
    #[arg(long, default_value = "http://localhost:8000")]
    server_url: String,
    
    /// Start server mode (run as background server)
    #[arg(long, conflicts_with = "use_server")]
    server: bool,
    
    /// Server port (when using --server)
    #[arg(long, default_value = "8000")]
    server_port: u16,
    
    /// Server host (when using --server)
    #[arg(long, default_value = "127.0.0.1")]
    server_host: String,
    
    /// Stop running server
    #[arg(long, conflicts_with_all = ["use_server", "server"])]
    server_stop: bool,
    
    /// Check server status
    #[arg(long, conflicts_with_all = ["use_server", "server", "server_stop"])]
    server_status: bool,
}

fn get_bridge_script_path() -> Result<PathBuf> {
    // Get the directory of the current executable
    let exe_path = std::env::current_exe()
        .context("Failed to get executable path")?;
    let exe_dir = exe_path.parent()
        .context("Failed to get executable directory")?;
    
    // Look for the bridge script in the same directory as the executable
    let bridge_script = exe_dir.join("easyocr_bridge.py");
    
    // If not found there, try the current working directory
    if !bridge_script.exists() {
        let cwd_bridge = PathBuf::from("easyocr_bridge.py");
        if cwd_bridge.exists() {
            return Ok(cwd_bridge);
        }
        
        // Try relative to current directory
        let relative_bridge = PathBuf::from("./easyocr_bridge.py");
        if relative_bridge.exists() {
            return Ok(relative_bridge);
        }
    }
    
    Ok(bridge_script)
}

fn run_ocr_subprocess(
    image_path: &str,
    languages: &[String],
    gpu: bool,
    detail: i32,
) -> Result<Vec<OcrResult>> {
    let bridge_script = get_bridge_script_path()
        .context("Failed to locate easyocr_bridge.py")?;
    
    if !bridge_script.exists() {
        anyhow::bail!(
            "Bridge script not found at: {}. Make sure easyocr_bridge.py is in the same directory as the binary.",
            bridge_script.display()
        );
    }
    
    // Build the command
    let langs = languages.join(",");
    let output = Command::new("python3")
        .arg(&bridge_script)
        .arg("--languages")
        .arg(&langs)
        .arg("--image")
        .arg(image_path)
        .arg("--gpu")
        .arg(gpu.to_string())
        .arg("--detail")
        .arg(detail.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to execute Python bridge script")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("OCR processing failed: {}", stderr);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Vec<OcrResult> = serde_json::from_str(&stdout)
        .context("Failed to parse OCR results from Python bridge")?;
    
    Ok(results)
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle server management commands
    if args.server_stop {
        return server::stop_server();
    }
    
    if args.server_status {
        if server::is_server_running() {
            println!("Server is running");
            if let Some(pid) = server::read_server_pid() {
                println!("PID: {}", pid);
            }
        } else {
            println!("Server is not running");
        }
        return Ok(());
    }
    
    if args.server {
        let child = server::start_server(args.server_port, &args.server_host)?;
        let pid = child.id();
        
        eprintln!("Server started successfully!");
        eprintln!("PID: {}", pid);
        eprintln!("URL: http://{}:{}", args.server_host, args.server_port);
        eprintln!("\nYou can now use the client:");
        eprintln!("  rustocr -i image.jpg --use-server\n");
        eprintln!("To stop the server:");
        eprintln!("  rustocr --server-stop\n");
        
        server::save_server_pid(pid)?;
        
        // Keep the server running (don't wait for it to finish)
        std::mem::forget(child);
        return Ok(());
    }

    // Validate input is provided for OCR operations
    let input = args.input.as_ref()
        .ok_or_else(|| anyhow::anyhow!("--input is required for OCR operations"))?;

    // Validate input file exists (only for OCR operations)
    if !Path::new(input).exists() {
        anyhow::bail!("Input file does not exist: {}", input);
    }

    // Validate detail level
    if args.detail != 0 && args.detail != 1 {
        anyhow::bail!("Detail level must be 0 or 1");
    }

    // Print initialization message
    eprintln!("Initializing OCR with languages: {:?}", args.languages);
    eprintln!("GPU enabled: {}", args.gpu);
    eprintln!("Processing image: {}", input);
    
    // Choose mode
    let results = if args.use_server {
        eprintln!("Using server mode at: {}", args.server_url);
        
        // Check server health
        if !check_server_health(&args.server_url)? {
            eprintln!("Warning: Server at {} is not responding", args.server_url);
            eprintln!("Make sure the server is running: python3 easyocr_server.py");
            anyhow::bail!("Server not available");
        }
        
        ocr_via_server(
            input,
            &args.languages,
            args.detail,
            args.gpu,
            &args.server_url
        )?
    } else {
        eprintln!("Using subprocess mode");
        run_ocr_subprocess(input, &args.languages, args.gpu, args.detail)?
    };

    // Output results based on format
    match args.output.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&results)
                .context("Failed to serialize results to JSON")?;
            println!("{}", json);
        }
        "text" => {
            for result in &results {
                println!("{}", result.text);
            }
        }
        "detailed" => {
            for (i, result) in results.iter().enumerate() {
                println!("--- Result {} ---", i + 1);
                println!("Text: {}", result.text);
                if args.detail == 1 && result.confidence > 0.0 {
                    println!("Confidence: {:.4}", result.confidence);
                    if !result.bbox.is_empty() {
                        println!("Bounding Box: {:?}", result.bbox);
                    }
                }
                println!();
            }
        }
        _ => {
            anyhow::bail!("Invalid output format. Use: json, text, or detailed");
        }
    }

    eprintln!("OCR completed. Found {} text region(s).", results.len());

    Ok(())
}
