use anyhow::Result;
use serde::Serialize;
use std::path::Path;

/// Output template system for flexible result formatting
/// 
/// Supports multiple output formats: CSV, JSON, XML, Markdown

#[derive(Debug, Serialize)]
pub struct OcrOutput {
    pub file: String,
    pub text: String,
    pub confidence: f32,
    pub bbox: Option<Vec<Vec<i32>>>,
}

/// Format output as CSV
pub fn format_csv(results: &[OcrOutput]) -> Result<String> {
    let mut output = String::from("file,text,confidence,bbox_x1,bbox_y1,bbox_x2,bbox_y2\n");
    
    for result in results {
        let bbox_str = if let Some(bbox) = &result.bbox {
            if !bbox.is_empty() {
                format!("{},{},{},{}", 
                    bbox[0][0], bbox[0][1], bbox[2][0], bbox[2][1])
            } else {
                ",,,,".to_string()
            }
        } else {
            ",,,,".to_string()
        };
        
        let text_escaped = result.text.replace('"', "\"\"");
        output.push_str(&format!(
            "\"{}\",\"{}\",{},{}\n",
            result.file, text_escaped, result.confidence, bbox_str
        ));
    }
    
    Ok(output)
}

/// Format output as XML
pub fn format_xml(results: &[OcrOutput]) -> Result<String> {
    let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<ocr_results>\n");
    
    for result in results {
        output.push_str("  <result>\n");
        output.push_str(&format!("    <file>{}</file>\n", result.file));
        output.push_str(&format!("    <text>{}</text>\n", 
            html_escape::encode_text(&result.text)));
        output.push_str(&format!("    <confidence>{}</confidence>\n", result.confidence));
        
        if let Some(bbox) = &result.bbox {
            output.push_str("    <bbox>\n");
            for (i, point) in bbox.iter().enumerate() {
                output.push_str(&format!(
                    "      <point{} x=\"{}\" y=\"{}\"/>\n",
                    i + 1, point[0], point[1]
                ));
            }
            output.push_str("    </bbox>\n");
        }
        
        output.push_str("  </result>\n");
    }
    
    output.push_str("</ocr_results>\n");
    Ok(output)
}

/// Format output as Markdown
pub fn format_markdown(results: &[OcrOutput]) -> Result<String> {
    let mut output = String::from("# OCR Results\n\n");
    
    for (i, result) in results.iter().enumerate() {
        output.push_str(&format!("## Result {}\n\n", i + 1));
        output.push_str(&format!("**File:** `{}`\n\n", result.file));
        output.push_str(&format!("**Text:**\n```\n{}\n```\n\n", result.text));
        output.push_str(&format!("**Confidence:** {:.2}%\n\n", result.confidence * 100.0));
        
        if let Some(bbox) = &result.bbox {
            output.push_str("**Bounding Box:**\n");
            for (i, point) in bbox.iter().enumerate() {
                output.push_str(&format!("- Point {}: ({}, {})\n", i + 1, point[0], point[1]));
            }
            output.push_str("\n");
        }
        
        output.push_str("---\n\n");
    }
    
    Ok(output)
}

/// Format output as JSON (default)
pub fn format_json(results: &[OcrOutput]) -> Result<String> {
    serde_json::to_string_pretty(results)
        .map_err(|e| anyhow::anyhow!("JSON formatting failed: {}", e))
}

/// Save output to file
pub fn save_output(content: &str, path: &Path) -> Result<()> {
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_json() {
        let results = vec![OcrOutput {
            file: "test.jpg".to_string(),
            text: "Hello".to_string(),
            confidence: 0.95,
            bbox: None,
        }];
        
        let json = format_json(&results).unwrap();
        assert!(json.contains("Hello"));
        assert!(json.contains("test.jpg"));
    }

    #[test]
    fn test_format_csv() {
        let results = vec![OcrOutput {
            file: "test.jpg".to_string(),
            text: "World".to_string(),
            confidence: 0.90,
            bbox: None,
        }];
        
        let csv = format_csv(&results).unwrap();
        assert!(csv.contains("file,text,confidence"));
        assert!(csv.contains("World"));
    }
}
