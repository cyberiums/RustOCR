use anyhow::{Context, Result};
use clap::Parser;
use rustocr::OcrReader;
use std::path::Path;

/// RustOCR - A fast Rust CLI for EasyOCR with 80+ language support
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input image file path
    #[arg(short, long)]
    input: String,

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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate input file exists
    if !Path::new(&args.input).exists() {
        anyhow::bail!("Input file does not exist: {}", args.input);
    }

    // Validate detail level
    if args.detail != 0 && args.detail != 1 {
        anyhow::bail!("Detail level must be 0 or 1");
    }

    // Print initialization message
    eprintln!("Initializing OCR reader with languages: {:?}", args.languages);
    eprintln!("GPU enabled: {}", args.gpu);

    // Create OCR reader
    let reader = OcrReader::new(args.languages.clone(), args.gpu)
        .context("Failed to initialize OCR reader")?;

    eprintln!("Processing image: {}", args.input);

    // Perform OCR
    let results = reader
        .read_text(&args.input, args.detail)
        .context("Failed to perform OCR")?;

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
                if args.detail == 1 {
                    println!("Confidence: {:.4}", result.confidence);
                    println!("Bounding Box: {:?}", result.bbox);
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
