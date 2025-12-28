# Server Mode Testing & README

## Server Mode v0.3.0 - Quick Start Guide

### Installation

1. Install server dependencies:
```bash
cd rust  
pip install -r requirements-server.txt
```

2. Build the Rust client:
```bash
cargo build --release
```

### Usage

#### Method 1: Manual Server Management

**Terminal 1 - Start Server:**
```bash
python3 easyocr_server.py --port 8000
```

**Terminal 2 - Use Client:**
```bash
# Server mode (fast for multiple requests)
./target/release/rustocr -i image.jpg --use-server

# Subprocess mode (default, no server needed)
./target/release/rustocr -i image.jpg
```

#### Method 2: Custom Server URL
```bash
# Start server on different port
python3 easyocr_server.py --port 9000

# Point client to custom URL
./target/release/rustocr -i image.jpg --use-server --server-url http://localhost:9000
```

### Server API Endpoints

- `GET /` - Server info
- `GET /api/v1/health` - Health check
- `GET /api/v1/models` - List loaded models
- `POST /api/v1/ocr` - Perform OCR
- `POST /api/v1/models/warmup` - Preload models

### Performance Comparison

| Mode | First Request | Subsequent Requests | Best For |
|------|---------------|---------------------|----------|
| Subprocess | 3-5s | 3-5s | Single images |
| Server | 3-5s | <500ms | Multiple images |

### Example: Batch Processing with Server

```bash
# Start server once
python3 easyocr_server.py &

# Process multiple images (each request is fast)
for img in images/*.jpg; do
    ./target/release/rustocr -i "$img" --use-server -o text
done

# Stop server
kill %1
```

### Troubleshooting

**Server won't start:**
- Check if port 8000 is already in use: `lsof -i :8000`
- Try a different port: `python3 easyocr_server.py --port 8001`

**Client can't connect:**
- Verify server is running: `curl http://localhost:8000/api/v1/health`
- Check firewall settings
- Ensure correct URL: `--server-url http://localhost:8000`

**Slow performance:**
- First request loads models (expected)
- Subsequent requests should be <500ms
- Check GPU is available: `nvidia-smi` (if using CUDA)

### Development

**Server Logs:**
```bash
# View detailed logs
python3 easyocr_server.py --reload  # Auto-reload on changes
```

**Client Debug:**
```bash
# Check connection
curl -X POST http://localhost:8000/api/v1/ocr \
  -H "Content-Type: application/json" \
  -d '{"image": "...", "languages": ["en"], "detail": 1, "gpu": true}'
```

## Testing Results

### Phase 1 & 2 Tests ✅

1. **Server Startup:** ✓ 
   - Server starts on port 8000
   - Health endpoint responds correctly
   
2. **CLI Flags:** ✓
   - `--use-server` flag available
   - `--server-url` flag with default value
   
3. **Server Health Check:** ✓
   - Client checks server before sending requests
   - Graceful error message if server unavailable

### Next Steps (Phase 3)

- [ ] Add `--server` flag to start server from Rust
- [ ] Implement background server process management  
- [ ] Add `--server-stop` command
- [ ] Full end-to-end OCR test with latency measurement
