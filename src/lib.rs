use anyhow::{Context, Result};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde::{Deserialize, Serialize};

/// Represents the result of OCR detection for a single text region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    /// Bounding box coordinates as [[x1, y1], [x2, y2], [x3, y3], [x4, y4]]
    pub bbox: Vec<Vec<i32>>,
    /// Detected text
    pub text: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
}

/// OCR Reader that wraps the EasyOCR Python library
pub struct OcrReader {
    reader: PyObject,
}

impl OcrReader {
    /// Create a new OCR reader with specified languages and GPU setting
    ///
    /// # Arguments
    /// * `languages` - List of language codes (e.g., ["en", "ch_sim"])
    /// * `gpu` - Whether to use GPU acceleration (requires CUDA)
    ///
    /// # Example
    /// ```no_run
    /// use rustocr::OcrReader;
    /// 
    /// let reader = OcrReader::new(vec!["en".to_string()], false).unwrap();
    /// ```
    pub fn new(languages: Vec<String>, gpu: bool) -> Result<Self> {
        Python::with_gil(|py| {
            // Import easyocr module
            let easyocr = py
                .import("easyocr")
                .context("Failed to import easyocr. Make sure it's installed: pip install easyocr")?;

            // Create language list
            let lang_list = PyList::new(py, &languages);

            // Create Reader instance
            let kwargs = PyDict::new(py);
            kwargs.set_item("gpu", gpu)?;
            kwargs.set_item("verbose", false)?;

            let reader = easyocr
                .getattr("Reader")?
                .call((lang_list,), Some(kwargs))
                .context("Failed to create EasyOCR Reader instance")?;

            Ok(OcrReader {
                reader: reader.into(),
            })
        })
    }

    /// Perform OCR on an image file
    ///
    /// # Arguments
    /// * `image_path` - Path to the image file
    /// * `detail` - Detail level: 0 for text only, 1 for full details
    ///
    /// # Returns
    /// * `Vec<OcrResult>` - List of detected text regions with bounding boxes and confidence scores
    ///
    /// # Example
    /// ```no_run
    /// use rustocr::OcrReader;
    /// 
    /// let reader = OcrReader::new(vec!["en".to_string()], false).unwrap();
    /// let results = reader.read_text("image.jpg", 1).unwrap();
    /// 
    /// for result in results {
    ///     println!("Text: {}, Confidence: {:.2}", result.text, result.confidence);
    /// }
    /// ```
    pub fn read_text(&self, image_path: &str, detail: i32) -> Result<Vec<OcrResult>> {
        Python::with_gil(|py| {
            let reader = self.reader.as_ref(py);

            // Call readtext method
            let kwargs = PyDict::new(py);
            kwargs.set_item("detail", detail)?;

            let result = reader
                .call_method("readtext", (image_path,), Some(kwargs))
                .context("Failed to perform OCR on image")?;

            // Parse results
            if detail == 0 {
                // Simple mode: just text strings
                let text_list: Vec<String> = result.extract()?;
                Ok(text_list
                    .into_iter()
                    .map(|text| OcrResult {
                        bbox: vec![],
                        text,
                        confidence: 0.0,
                    })
                    .collect())
            } else {
                // Detailed mode: (bbox, text, confidence) tuples
                let results_list: &PyList = result
                    .downcast()
                    .map_err(|e| anyhow::anyhow!("Failed to parse OCR results: {}", e))?;
                let mut ocr_results = Vec::new();

                for item in results_list.iter() {
                    let tuple: &pyo3::types::PyTuple = item
                        .downcast()
                        .map_err(|e| anyhow::anyhow!("Failed to parse result tuple: {}", e))?;

                    // Extract bbox (list of [x, y] coordinates)
                    let bbox_list: &PyList = tuple
                        .get_item(0)?
                        .downcast()
                        .map_err(|e| anyhow::anyhow!("Failed to parse bounding box: {}", e))?;
                    let mut bbox = Vec::new();
                    for coord in bbox_list.iter() {
                        let coord_list: &PyList = coord
                            .downcast()
                            .map_err(|e| anyhow::anyhow!("Failed to parse coordinates: {}", e))?;
                        let x: i32 = coord_list.get_item(0)?.extract()?;
                        let y: i32 = coord_list.get_item(1)?.extract()?;
                        bbox.push(vec![x, y]);
                    }

                    // Extract text
                    let text: String = tuple.get_item(1)?.extract()?;

                    // Extract confidence
                    let confidence: f64 = tuple.get_item(2)?.extract()?;

                    ocr_results.push(OcrResult {
                        bbox,
                        text,
                        confidence,
                    });
                }

                Ok(ocr_results)
            }
        })
    }

    /// Perform OCR and return results as simple text strings
    ///
    /// # Arguments
    /// * `image_path` - Path to the image file
    ///
    /// # Returns
    /// * `Vec<String>` - List of detected text strings
    pub fn read_text_simple(&self, image_path: &str) -> Result<Vec<String>> {
        let results = self.read_text(image_path, 0)?;
        Ok(results.into_iter().map(|r| r.text).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires EasyOCR to be installed
    fn test_reader_creation() {
        let reader = OcrReader::new(vec!["en".to_string()], false);
        assert!(reader.is_ok());
    }
}
