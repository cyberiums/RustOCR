# API Endpoints Documentation

## EasyOCR2 REST API

This document provides examples for all available API endpoints.

### Base URL
```
http://localhost:8000
```

### API Documentation
- Swagger UI: `http://localhost:8000/docs`
- ReDoc: `http://localhost:8000/redoc`

---

## Endpoints

### 1. Root Endpoint
**GET /**

Returns basic server information.

```bash
curl http://localhost:8000/
```

Response:
```json
{
  "name": "EasyOCR2 API Server",
  "version": "1.0.0",
  "status": "running",
  "docs": "/docs",
  "endpoints": {
    "ocr": "/api/v1/ocr",
    "health": "/api/v1/health",
    "info": "/api/v1/info",
    "stats": "/api/v1/stats",
    "models": "/api/v1/models"
  }
}
```

---

### 2. Health Check
**GET /api/v1/health**

Check server health and loaded models.

```bash
curl http://localhost:8000/api/v1/health
```

Response:
```json
{
  "status": "healthy",
  "models_loaded": ["en", "ch_sim,en"],
  "uptime_seconds": 3600.5
}
```

---

### 3. Server Information  
**GET /api/v1/info**

Get detailed server information.

```bash
curl http://localhost:8000/api/v1/info
```

Response:
```json
{
  "name": "EasyOCR2 API Server",
  "version": "1.0.0",
  "description": "High-performance OCR API with 80+ language support",
  "supported_languages": 80,
  "gpu_available": true,
  "uptime_seconds": 3600.5,
  "endpoints": {
    "docs": "/docs",
    "ocr": "/api/v1/ocr",
    "health": "/api/v1/health",
    ...
  }
}
```

---

### 4. Server Statistics
**GET /api/v1/stats**

Get server usage statistics.

```bash
curl http://localhost:8000/api/v1/stats
```

Response:
```json
{
  "total_requests": 1250,
  "successful_requests": 1200,
  "failed_requests": 50,
  "success_rate": 96.0,
  "average_processing_time_ms": 450.75,
  "loaded_models": ["en", "ch_sim,en"],
  "uptime_seconds": 3600.5
}
```

---

### 5. List Models
**GET /api/v1/models**

List currently loaded models.

```bash
curl http://localhost:8000/api/v1/models
```

Response:
```json
{
  "models": {
    "en": {
      "languages": ["en"],
      "loaded_at": 1703779200.5,
      "gpu_enabled": true
    }
  },
  "count": 1
}
```

---

### 6. Perform OCR
**POST /api/v1/ocr**

Perform OCR on a base64-encoded image.

```bash
# Using JSON
curl -X POST http://localhost:8000/api/v1/ocr \
  -H "Content-Type: application/json" \
  -d '{
    "image": "<base64-encoded-image>",
    "languages": ["en"],
    "detail": 1,
    "gpu": true
  }'
```

Request Body:
```json
{
  "image": "iVBORw0KGgoAAAANSUhEUg...",
  "languages": ["en"],
  "detail": 1,
  "gpu": true
}
```

Response:
```json
{
  "status": "success",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "results": [
    {
      "bbox": [[10, 20], [100, 20], [100, 50], [10, 50]],
      "text": "Hello World",
      "confidence": 0.9872
    }
  ],
  "processing_time_ms": 450.5,
  "model_load_time_ms": 0.2,
  "timestamp": "2025-12-28T11:30:00.000000"
}
```

---

### 7. Warmup Model
**POST /api/v1/models/warmup**

Preload models for specific languages.

```bash
curl -X POST "http://localhost:8000/api/v1/models/warmup?languages=en&languages=ch_sim&gpu=true"
```

Response:
```json
{
  "status": "success",
  "languages": ["en", "ch_sim"]
}
```

---

## Error Responses

All endpoints return standardized error responses:

```json
{
  "detail": "Error message here"
}
```

HTTP Status Codes:
- `200` - Success
- `400` - Bad Request (invalid image data, etc.)
- `500` - Internal Server Error

---

## Integration Examples

### Python
```python
import requests
import base64

# Read and encode image
with open('image.jpg', 'rb') as f:
    image_b64 = base64.b64encode(f.read()).decode()

# Perform OCR
response = requests.post('http://localhost:8000/api/v1/ocr', json={
    'image': image_b64,
    'languages': ['en'],
    'detail': 1,
    'gpu': True
})

results = response.json()
print(results['results'])
```

### JavaScript
```javascript
const response = await fetch('http://localhost:8000/api/v1/ocr', {
  method: 'POST',
  headers: {'Content-Type': 'application/json'},
  body: JSON.stringify({
    image: base64Image,
    languages: ['en'],
    detail: 1,
    gpu: true
  })
});

const data = await response.json();
console.log(data.results);
```

### cURL with File
```bash
# Convert image to base64
base64_image=$(base64 -i image.jpg)

# Call API
curl -X POST http://localhost:8000/api/v1/ocr \
  -H "Content-Type: application/json" \
  -d "{\"image\": \"$base64_image\", \"languages\": [\"en\"]}"
```
