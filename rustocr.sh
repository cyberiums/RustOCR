#!/bin/bash
# Wrapper script for rustocr that sets up the Python environment

# Get Python library path
PYTHON_LIB_DIR=$(python3 -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")

# Set library path for macOS
export DYLD_LIBRARY_PATH="${PYTHON_LIB_DIR}:${DYLD_LIBRARY_PATH}"

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Run the actual binary
exec "${SCRIPT_DIR}/target/release/rustocr" "$@"
