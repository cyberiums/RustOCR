use anyhow::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Parallel processing module for batch OCR operations
/// 
/// Uses rayon for CPU parallelization to process multiple images concurrently

#[derive(Debug)]
pub struct ParallelConfig {
    pub num_threads: usize,
    pub chunk_size: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_threads: num_cpus::get(),
            chunk_size: 10,
        }
    }
}

/// Process images in parallel using rayon
pub fn process_images_parallel<F>(
    images: Vec<PathBuf>,
    config: ParallelConfig,
    processor: F,
) -> Vec<Result<String>>
where
    F: Fn(&PathBuf) -> Result<String> + Send + Sync,
{
    // Configure rayon thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.num_threads)
        .build()
        .expect("Failed to build thread pool");

    // Process images in parallel
    images
        .par_iter()
        .map(|img| processor(img))
        .collect()
}

/// Process with progress tracking
pub fn process_images_parallel_with_progress<F>(
    images: Vec<PathBuf>,
    config: ParallelConfig,
    processor: F,
) -> Vec<Result<String>>
where
    F: Fn(&PathBuf) -> Result<String> + Send + Sync,
{
    use indicatif::{ProgressBar, ProgressStyle};
    
    let pb = Arc::new(Mutex::new(ProgressBar::new(images.len() as u64)));
    {
        let pb = pb.lock().unwrap();
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("█▓▒░  "),
        );
    }

    let results: Vec<Result<String>> = images
        .par_iter()
        .map(|img| {
            let result = processor(img);
            pb.lock().unwrap().inc(1);
            result
        })
        .collect();

    pb.lock().unwrap().finish_with_message("Parallel processing complete");
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.num_threads > 0);
        assert_eq!(config.chunk_size, 10);
    }
}
