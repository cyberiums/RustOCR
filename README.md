# RustOCR

**Fastly Built by [FastBuilder.AI](https://fastbuilder.ai)** 🚀

A fast Rust CLI wrapper for EasyOCR providing OCR capabilities with 80+ language support.

## 🎯 Quick Start (Recommended for FastBuilder.AI Integration)

### Direct Python Bridge Usage (ARM64/Apple Silicon Compatible)

The recommended way to use RustOCR in automated systems is to call the Python bridge directly:

```bash
python3 easyocr_bridge.py --languages en --image /path/to/image.png --gpu false --detail 1
```

**Why this approach?**
- ✅ Works on all CPU architectures (x86_64, ARM64/Apple Silicon)
- ✅ No binary compilation needed
- ✅ Direct access to EasyOCR functionality
- ✅ Simpler error handling

### Example Integration (Rust)

```rust
use std::process::Command;

let output = Command::new("python3")
    .arg("/path/to/easyocr_bridge.py")
    .arg("--languages").arg("en")
    .arg("--image").arg("/path/to/image.png")
    .arg("--gpu").arg("false")
    .arg("--detail").arg("1")
    .output()
    .expect("Failed to run OCR");

let json_str = String::from_utf8_lossy(&output.stdout);
let results: Vec<OcrResult> = serde_json::from_str(&json_str)?;
```

## ✨ Complete Feature List

### Core Features
- ✅ **Basic OCR** - Image to text conversion
- ✅ **80+ Languages** - Multi-language support
- ✅ **GPU/CPU Support** - Flexible processing
- ✅ **Confidence Filtering** - Quality control

### Performance Features
- ✅ **Server Mode** - 5-10x faster with persistent server
- ✅ **Batch Processing** - Process multiple images
- ✅ **Parallel Processing** - Multi-core batch processing
- ✅ **Progress Indicators** - Visual feedback

### Usability Features
- ✅ **Configuration Files** - TOML-based settings
- ✅ **Named Profiles** - Reusable workflows
- ✅ **Watch Mode** - Automatic directory monitoring
- ✅ **Output Templates** - CSV, XML, Markdown, JSON

### API Features
- ✅ **Enhanced REST API** - Production-ready endpoints
- ✅ **OpenAPI Documentation** - /docs and /redoc
- ✅ **Server Statistics** - Usage monitoring
- ✅ **CORS Support** - Web integration

##  Prerequisites

Before using RustOCR, ensure you have:

1. **Python 3.7+** installed
2. **EasyOCR** Python library:
   ```bash
   pip install easyocr
   ```

   > **Note:** If you encounter import errors with EasyOCR (e.g., `bidi` module issues), try reinstalling:
   > ```bash
   > pip install --upgrade --force-reinstall python-bidi easyocr
   > ```

3. **Rust toolchain** (only for building the binary):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

## Installation & Usage

### Method 1: Python Bridge (Recommended)

**Setup:**
```bash
# Ensure easyocr_bridge.py is executable
chmod +x easyocr_bridge.py

# Test it
python3 easyocr_bridge.py --languages en --image test.png --gpu false --detail 1
```

**Python Bridge Arguments:**
```
--languages \u003cLANGS\u003e    Comma-separated language codes (e.g., "en" or "ch_sim,en")
--image \u003cFILE\u003e         Path to image file
--gpu \u003ctrue|false\u003e     Enable GPU acceleration (default: true)
--detail \u003c0|1\u003e         Detail level: 0 for text only, 1 for bbox + confidence
```

**Output Format (JSON):**
```json
[
  {
    "bbox": [[29, 11], [279, 11], [279, 29], [29, 29]],
    "text": "Error Agent execution terminated",
    "confidence": 0.9144445
  },
  {
    "bbox": [[405, 187], [439, 187], [439, 203], [405, 203]],
    "text": "Retry",
    "confidence": 0.9999841
  }
]
```

### Method 2: Compiled Binary (x86_64 Only)

> ⚠️ **Known Issue:** Pre-compiled binaries may have CPU architecture mismatches on ARM64/Apple Silicon systems. Use Python bridge instead.

**If building from source:**
```bash
cargo build --release
```

**Usage with wrapper script:**
```bash
./rustocr.sh -i image.jpg -l en -o json
```

The `rustocr.sh` wrapper script:
- Sets up Python library paths
- Handles OpenCV dependencies
- Runs the compiled binary with proper environment

## Architecture

### Current Implementation (Subprocess Mode)

```
┌─────────────────┐
│   Your Code     │ (Rust/Python/Shell)
└────────┬────────┘
         │
         ↓ subprocess
┌─────────────────┐
│ easyocr_bridge  │ (Python script)
│     (CLI)       │
└────────┬────────┘
         │
         ↓ import
┌─────────────────┐
│    EasyOCR      │ (Python library)
│   (PyTorch)     │
└─────────────────┘
```

### Production Deployment Recommendations

1. **For Automated Systems (FastBuilder.AI):**
   - Use Python bridge directly
   - Set `--gpu false` for CPU-only processing (faster startup, no CUDA dependency)
   - Cache the EasyOCR reader instance in server mode for better performance

2. **For Interactive CLI:**
   - Build rustocr binary for your specific architecture
   - Use wrapper script for proper environment setup

3. **For High-Throughput:**
   - Use server mode (see SERVER_MODE.md)
   - Keep persistent EasyOCR reader loaded
   - Process requests via REST API

## Supported Languages

EasyOCR supports 80+ languages including:

- **Latin**: English (en), French (fr), German (de), Spanish (es), Portuguese (pt), Italian (it)
- **Chinese**: Simplified (ch_sim), Traditional (ch_tra)
- **Japanese** (ja), **Korean** (ko), **Thai** (th)
- **Arabic** (ar), **Persian** (fa), **Urdu** (ur)
- **Cyrillic**: Russian (ru), Ukrainian (uk), Bulgarian (bg)
- **Devanagari**: Hindi (hi), Marathi (mr), Nepali (ne)

See the [full list of supported languages](https://github.com/cyberiums/EasyOCR).

## Troubleshooting

### "Bad CPU type in executable"
**Problem:** Pre-compiled binary doesn't match your CPU architecture.

**Solution:** Use Python bridge directly:
```bash
python3 easyocr_bridge.py --languages en --image image.png --gpu false --detail 1
```

### "Bridge script not found"
**Problem:** rustocr binary can't find easyocr_bridge.py.

**Solution:** 
1. Ensure easyocr_bridge.py is in the same directory as rustocr binary
2. Or use Python bridge directly (see above)

### "Failed to import easyocr"
**Problem:** EasyOCR Python library not installed.

**Solution:**
```bash
pip install easyocr
# Or force reinstall if issues persist
pip install --upgrade --force-reinstall python-bidi easyocr
```

### GPU-related errors
**Problem:** CUDA/GPU errors or missing dependencies.

**Solution:** Use CPU mode:
```bash
python3 easyocr_bridge.py --languages en --image image.png --gpu false --detail 1
```

## Performance Tips

- **First Run:** EasyOCR downloads models on first use (~100MB). Subsequent runs are fast.
- **GPU vs CPU:** GPU is faster but CPU mode works fine for occasional OCR tasks
- **Server Mode:** For high throughput, use server mode to keep models loaded in memory
- **Language Selection:** Only specify languages you need for faster initialization

## Integration Examples

### FastBuilder.AI LocalApp (Rust)
```rust
// In localApp/src-tauri/src/ocr.rs
let bridge_path = "/path/to/easyocr_bridge.py";
let output = Command::new("python3")
    .arg(bridge_path)
    .arg("--languages").arg("en")
    .arg("--image").arg(screenshot_path)
    .arg("--gpu").arg("false")
    .arg("--detail").arg("1")
    .output()?;

let results: Vec<OcrResult> = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
```

### Shell Script
```bash
#!/bin/bash
RESULT=$(python3 easyocr_bridge.py \
    --languages en \
    --image screenshot.png \
    --gpu false \
    --detail 1)

echo "$RESULT" | jq '.[] | select(.text | contains("Retry"))'
```

### Python
```python
import subprocess
import json

result = subprocess.run([
    "python3", "easyocr_bridge.py",
    "--languages", "en",
    "--image", "screenshot.png",
    "--gpu", "false",
    "--detail", "1"
], capture_output=True, text=True)

data = json.loads(result.stdout)
for item in data:
    if "retry" in item["text"].lower():
        print(f"Found Retry button at {item['bbox']}")
```

## License

Apache-2.0 (matching the parent EasyOCR project)

## Credits

- [EasyOCR2](https://github.com/cyberiums/EasyOCR) - The underlying OCR engine
- Built with [PyO3](https://pyo3.rs/) and [Clap](https://docs.rs/clap/)
- Maintained by [FastBuilder.AI](https://fastbuilder.ai)
