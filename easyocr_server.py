#!/usr/bin/env python3
"""
EasyOCR2 FastAPI Server for rustocr
Provides persistent OCR service with model caching for improved performance
"""
import asyncio
import base64
import io
import logging
import time
from typing import List, Optional
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException, UploadFile, File
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import uvicorn
from PIL import Image
import easyocr  # TODO: Change to easyocr2 when package is published

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Global model cache
MODEL_CACHE = {}


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


class HealthResponse(BaseModel):
    """Health check response"""
    status: str
    models_loaded: List[str]
    uptime_seconds: float


class ModelInfo(BaseModel):
    """Model information"""
    languages: List[str]
    loaded_at: float
    gpu_enabled: bool


# Server startup time
START_TIME = time.time()


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
    logger.info("Starting EasyOCR2 FastAPI Server")
    logger.info("Server ready to accept requests")
    yield
    # Shutdown
    logger.info("Shutting down server")
    MODEL_CACHE.clear()


# Create FastAPI app
app = FastAPI(
    title="EasyOCR2 Server",
    description="High-performance OCR server with model caching",
    version="0.3.0",
    lifespan=lifespan
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/", response_model=dict)
async def root():
    """Root endpoint"""
    return {
        "name": "EasyOCR2 Server",
        "version": "0.3.0",
        "status": "running",
        "endpoints": {
            "ocr": "/api/v1/ocr",
            "health": "/api/v1/health",
            "models": "/api/v1/models"
        }
    }


@app.get("/api/v1/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint"""
    uptime = time.time() - START_TIME
    models_loaded = list(MODEL_CACHE.keys())
    
    return HealthResponse(
        status="healthy",
        models_loaded=models_loaded,
        uptime_seconds=uptime
    )


@app.get("/api/v1/models", response_model=dict)
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


@app.post("/api/v1/ocr", response_model=OcrResponse)
async def perform_ocr(request: OcrRequest):
    """
    Perform OCR on base64 encoded image
    
    Returns detected text with bounding boxes and confidence scores
    """
    import uuid
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
        
        logger.info(f"Request {request_id}: Completed in {total_time:.2f}ms "
                   f"(OCR: {ocr_time:.2f}ms, Model load: {model_load_time:.2f}ms)")
        
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


@app.post("/api/v1/models/warmup")
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
        "easyocr_server:app",
        host=args.host,
        port=args.port,
        workers=args.workers,
        reload=args.reload,
        log_level="info"
    )
