FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY src ./src
COPY easyocr_bridge.py easyocr_server.py easyocr_server_enhanced.py ./

# Build release binary
RUN cargo build --release

# Runtime stage
FROM python:3.11-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libgomp1 \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender-dev \
    libgl1-mesa-glx \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies
RUN pip install --no-cache-dir \
    easyocr2 \
    fastapi>=0.104.0 \
    uvicorn[standard]>=0.24.0 \
    python-multipart>=0.0.6 \
    pydantic>=2.0.0

# Copy binary from builder
COPY --from=builder /app/target/release/rustocr /usr/local/bin/

# Copy Python scripts
COPY --from=builder /app/easyocr_bridge.py /usr/local/bin/
COPY --from=builder /app/easyocr_server.py /usr/local/bin/
COPY --from=builder /app/easyocr_server_enhanced.py /usr/local/bin/

# Set working directory
WORKDIR /data

# Expose server port
EXPOSE 8000

# Default command
CMD ["rustocr", "--help"]
