use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents the result of OCR detection for a single text region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    /// Bounding box coordinates as [[x1, y1], [x2, y2], [x3, y3], [x4, y4]]
    #[serde(default)]
    pub bbox: Vec<Vec<i32>>,
    /// Detected text
    pub text: String,
    /// Confidence score (0.0 to 1.0)
    #[serde(default)]
    pub confidence: f64,
}

/// Request to server
#[derive(Debug, Serialize)]
struct OcrRequest {
    image: String,
    languages: Vec<String>,
    detail: i32,
    gpu: bool,
}

/// Response from server
#[derive(Debug, Deserialize)]
struct OcrResponse {
    status: String,
    request_id: String,
    results: Vec<OcrResult>,
    processing_time_ms: f64,
    model_load_time_ms: f64,
}

/// Perform OCR using the server
pub fn ocr_via_server(
    image_path: &str,
    languages: &[String],
    detail: i32,
    gpu: bool,
    server_url: &str,
) -> Result<Vec<OcrResult>> {
    // Read and encode image
    let image_data = std::fs::read(image_path)
        .context(format!("Failed to read image file: {}", image_path))?;
    let base64_image = base64::encode(&image_data);
    
    // Build request
    let request = OcrRequest {
        image: base64_image,
        languages: languages.to_vec(),
        detail,
        gpu,
    };
    
    // Send HTTP request
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/api/v1/ocr", server_url);
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .context(format!("Failed to connect to server at {}", server_url))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("Server returned error {}: {}", status, error_text);
    }
    
    // Parse response
    let ocr_response: OcrResponse = response
        .json()
        .context("Failed to parse server response")?;
    
    eprintln!("Server processing time: {:.2}ms (Model load: {:.2}ms)", 
             ocr_response.processing_time_ms, 
             ocr_response.model_load_time_ms);
    
    Ok(ocr_response.results)
}

/// Check if server is healthy
pub fn check_server_health(server_url: &str) -> Result<bool> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/api/v1/health", server_url);
    
    match client.get(&url).send() {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}
