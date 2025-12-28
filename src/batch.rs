use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use serde_json;
use crate::client::{ocr_via_server, OcrResult};

/// Batch result for one image
#[derive(Debug, serde::Serialize)]
pub struct BatchResult {
    pub file: String,
    pub success: bool,
    pub results: Option<Vec<OcrResult>>,
    pub error: Option<String>,
}

/// Process multiple images with progress indicator
pub fn process_batch(
    files: &[String],
    languages: &[String],
    detail: i32,
    gpu: bool,
    use_server: bool,
    server_url: &str,
    run_ocr_subprocess: impl Fn(&str, &[String], bool, i32) -> Result<Vec<OcrResult>>,
) -> Vec<BatchResult> {
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut results = Vec::new();

    for file in files {
        pb.set_message(format!("Processing {}", file));

        let result = if use_server {
            match ocr_via_server(file, languages, detail, gpu, server_url) {
                Ok(ocr_results) => BatchResult {
                    file: file.clone(),
                    success: true,
                    results: Some(ocr_results),
                    error: None,
                },
                Err(e) => BatchResult {
                    file: file.clone(),
                    success: false,
                    results: None,
                    error: Some(e.to_string()),
                },
            }
        } else {
            match run_ocr_subprocess(file, languages, gpu, detail) {
                Ok(ocr_results) => BatchResult {
                    file: file.clone(),
                    success: true,
                    results: Some(ocr_results),
                    error: None,
                },
                Err(e) => BatchResult {
                    file: file.clone(),
                    success: false,
                    results: None,
                    error: Some(e.to_string()),
                },
            }
        };

        results.push(result);
        pb.inc(1);
    }

    pb.finish_with_message("Batch processing complete");
    results
}
