#!/usr/bin/env python3
"""
EasyOCR2 FastAPI Server - Enhanced REST API
High-performance OCR server with comprehensive API endpoints
"""
import asyncio
import base64
import io
import logging
import time
import uuid
from typing import List, Optional
from contextlib import asynccontextmanager
from datetime import datetime

from fastapi import FastAPI, HTTPException, UploadFile, File, Request
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import uvicorn
from PIL import Image
import easyocr2 as easyocr

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Global model cache and statistics
MODEL_CACHE = {}
STATS = {
    'total_requests': 0,
    'successful_requests': 0,
    'failed_requests': 0,
    'processing_times': []
}
START_TIME = time.time()


class OcrRequest(BaseModel):
    """Request model for OCR operations"""
    image: str = Field(..., description="Base64 encoded image data")
    languages: List[str] = Field(default=["en"], description="Language codes")
    detail: int = Field(default=1, ge=0, le=1, description="Detail level: 0 or 1")
    gpu: bool = Field(default=True, description="Use GPU if available")


class OcrResult(BaseModel):
    """Single OCR result"""
    bbox: List[List[int]] = Field(default_factory=list)
    text: str
    confidence: float = 0.0


class OcrResponse(BaseModel):
    """Response model for OCR operations"""
    status: str
    request_id: str
    results: List[OcrResult]
    processing_time_ms: float
    model_load_time_ms: float
    timestamp: str = Field(default_factory=lambda: datetime.now().isoformat())


class HealthResponse(BaseModel):
    """Health check response"""
    status: str
    models_loaded: List[str]
    uptime_seconds: float


class ServerInfoResponse(BaseModel):
    """Server information response"""
    name: str = Field(description="Server name")
    version: str = Field(description="Server version")
    description: str = Field(description="Server description")
    supported_languages: int = Field(description="Number of supported languages")
    gpu_available: bool = Field(description="GPU availability")
    uptime_seconds: float = Field(description="Server uptime in seconds")
    endpoints: dict = Field(description="Available API endpoints")


class ServerStatsResponse(BaseModel):
    """Server statistics response"""
    total_requests: int = Field(description="Total requests processed")
    successful_requests: int = Field(description="Successful requests")
    failed_requests: int = Field(description="Failed requests")
    success_rate: float = Field(description="Success rate percentage")
    average_processing_time_ms: float = Field(description="Average processing time")
    loaded_models: List[str] = Field(description="Currently loaded models")
    uptime_seconds: float = Field(description="Server uptime")


class ModelInfo(BaseModel):
    """Model information"""
    languages: List[str]
    loaded_at: float
    gpu_enabled: bool


def get_model_key(languages: List[str]) -> str:
    """Generate cache key for language combination"""
    return ",".join(sorted(languages))


def get_or_create_model(languages: List[str], gpu: bool = True) -> easyocr.Reader:
    """Get model from cache or create new one"""
    model_key = get_model_key(languages)
    
    if model_key not in MODEL_CACHE:
        logger.info(f"Loading model for languages: {languages}")
        start_time = time.time()
        
        try:
            reader = easyocr.Reader(languages, gpu=gpu, verbose=False)
            MODEL_CACHE[model_key] = {
                'reader': reader,
                'languages': languages,
                'loaded_at': time.time(),
                'gpu_enabled': gpu
            }
            load_time = time.time() - start_time
            logger.info(f"Model loaded in {load_time:.2f}s")
        except Exception as e:
            logger.error(f"Failed to load model: {e}")
            raise HTTPException(status_code=500, detail=f"Failed to load model: {str(e)}")
    
    return MODEL_CACHE[model_key]['reader']


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan context manager for startup/shutdown"""
    # Startup
    logger.info("Starting EasyOCR2 FastAPI Server v1.0.0")
    logger.info("Server ready to accept requests")
    yield
    # Shutdown
    logger.info("Shutting down server")
    MODEL_CACHE.clear()


# Create FastAPI app with enhanced metadata
app = FastAPI(
    title="EasyOCR2 API Server",
    description="""
    High-performance OCR API with 80+ language support powered by EasyOCR2.
    
    ## Features
    - 🚀 5-10x faster with persistent model caching
    - 🌍 Support for 80+ languages
    - 💻 GPU acceleration
    - 📊 Request tracking and statistics
    - 🔧 Production-ready REST API
    
    ## Quick Start
    ```bash
    curl -X POST "http://localhost:8000/api/v1/ocr" \\
      -H "Content-Type: application/json" \\
      -d '{"image": "<base64-image>", "languages": ["en"]}'
    ```
    """,
    version="1.0.0",
    docs_url="/docs",
    redoc_url="/redoc",
    lifespan=lifespan,
    openapi_tags=[
        {"name": "OCR", "description": "Optical Character Recognition operations"},
        {"name": "Models", "description": "Model management and warmup"},
        {"name": "Server", "description": "Server health, info, and statistics"},
    ]
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Request tracking middleware
@app.middleware("http")
async def track_requests(request: Request, call_next):
    """Track request statistics"""
    request_id = str(uuid.uuid4())
    request.state.request_id = request_id
    
    start_time = time.time()
    
    try:
        response = await call_next(request)
        STATS['successful_requests'] += 1
        return response
    except Exception as e:
        STATS['failed_requests'] += 1
        raise
    finally:
        STATS['total_requests'] += 1
        process_time = (time.time() - start_time) * 1000
        STATS['processing_times'].append(process_time)
        if len(STATS['processing_times']) > 1000:  # Keep last 1000
            STATS['processing_times'].pop(0)


@app.get("/", response_model=dict, tags=["Server"])
async def root():
    """Root endpoint with server information"""
    return {
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


@app.get("/api/v1/health", response_model=HealthResponse, tags=["Server"])
async def health_check():
    """Health check endpoint"""
    uptime = time.time() - START_TIME
    models_loaded = list(MODEL_CACHE.keys())
    
    return HealthResponse(
        status="healthy",
        models_loaded=models_loaded,
        uptime_seconds=uptime
    )


@app.get("/api/v1/info", response_model=ServerInfoResponse, tags=["Server"])
async def get_server_info():
    """Get detailed server information"""
    import torch
    
    uptime = time.time() - START_TIME
    
    return ServerInfoResponse(
        name="EasyOCR2 API Server",
        version="1.0.0",
        description="High-performance OCR API with 80+ language support",
        supported_languages=80,  # EasyOCR supports 80+ languages
        gpu_available=torch.cuda.is_available() if hasattr(torch, 'cuda') else False,
        uptime_seconds=uptime,
        endpoints={
            "docs": "/docs",
            "ocr": "/api/v1/ocr",
            "health": "/api/v1/health",
            "info": "/api/v1/info",
            "stats": "/api/v1/stats",
            "models": "/api/v1/models",
            "warmup": "/api/v1/models/warmup"
        }
    )


@app.get("/api/v1/stats", response_model=ServerStatsResponse, tags=["Server"])
async def get_server_stats():
    """Get server statistics"""
    avg_time = sum(STATS['processing_times']) / len(STATS['processing_times']) if STATS['processing_times'] else 0
    total = STATS['total_requests']
    success_rate = (STATS['successful_requests'] / total * 100) if total > 0 else 0
    uptime = time.time() - START_TIME
    
    return ServerStatsResponse(
        total_requests=STATS['total_requests'],
        successful_requests=STATS['successful_requests'],
        failed_requests=STATS['failed_requests'],
        success_rate=round(success_rate, 2),
        average_processing_time_ms=round(avg_time, 2),
        loaded_models=list(MODEL_CACHE.keys()),
        uptime_seconds=round(uptime, 2)
    )


@app.get("/api/v1/models", response_model=dict, tags=["Models"])
async def list_models():
    """List loaded models"""
    models = {}
    for key, value in MODEL_CACHE.items():
        models[key] = ModelInfo(
            languages=value['languages'],
            loaded_at=value['loaded_at'],
            gpu_enabled=value['gpu_enabled']
        )
    return {"models": models, "count": len(models)}


@app.post("/api/v1/ocr", response_model=OcrResponse, tags=["OCR"])
async def perform_ocr(request: OcrRequest):
    """
    Perform OCR on base64 encoded image
    
    Returns detected text with bounding boxes and confidence scores
    """
    request_id = str(uuid.uuid4())
    
    logger.info(f"Request {request_id}: OCR for languages {request.languages}")
    start_time = time.time()
    
    try:
        # Decode base64 image
        try:
            image_data = base64.b64decode(request.image)
            image = Image.open(io.BytesIO(image_data))
        except Exception as e:
            raise HTTPException(status_code=400, detail=f"Invalid image data: {str(e)}")
        
        # Get or create model
        model_start = time.time()
        reader = get_or_create_model(request.languages, request.gpu)
        model_load_time = (time.time() - model_start) * 1000
        
        # Perform OCR
        ocr_start = time.time()
        results = reader.readtext(image, detail=request.detail)
        ocr_time = (time.time() - ocr_start) * 1000
        
        # Parse results
        ocr_results = []
        if request.detail == 0:
            # Simple mode: just text
            ocr_results = [OcrResult(text=text) for text in results]
        else:
            # Detailed mode: bbox, text, confidence
            for bbox, text, confidence in results:
                ocr_results.append(OcrResult(
                    bbox=[[int(x), int(y)] for x, y in bbox],
                    text=text,
                    confidence=float(confidence)
                ))
        
        total_time = (time.time() - start_time) * 1000
        
        logger.info(f"Request {request_id}: Completed in {total_time:.2f}ms")
        
        return OcrResponse(
            status="success",
            request_id=request_id,
            results=ocr_results,
            processing_time_ms=total_time,
            model_load_time_ms=model_load_time
        )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Request {request_id}: Error - {str(e)}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/api/v1/models/warmup", tags=["Models"])
async def warmup_model(languages: List[str], gpu: bool = True):
    """Preload model for specific languages"""
    try:
        logger.info(f"Warming up model for languages: {languages}")
        get_or_create_model(languages, gpu)
        return {"status": "success", "languages": languages}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="EasyOCR2 FastAPI Server")
    parser.add_argument("--host", default="127.0.0.1", help="Host to bind to")
    parser.add_argument("--port", type=int, default=8000, help="Port to bind to")
    parser.add_argument("--workers", type=int, default=1, help="Number of workers")
    parser.add_argument("--reload", action="store_true", help="Enable auto-reload")
    
    args = parser.parse_args()
    
    logger.info(f"Starting server on {args.host}:{args.port}")
    
    uvicorn.run(
        "easyocr_server_enhanced:app",
        host=args.host,
        port=args.port,
        workers=args.workers,
        reload=args.reload,
        log_level="info"
    )
