#!/usr/bin/env/bash
set -e

echo "==> Setting Standalone Python for Linux..."

RUNTIME_DIR="src-tauri/python-runtime"
mkdir -p "$RUNTIME_DIR"

curl -L https://github.com/astral-sh/python-build-standalone/releases/download/20261021/cpython-3.14.9+20261021-x86_64-unknown-linux-gnu-install_only.tar.gz -o python.tar.gz
tar -xzf python.tar.gz -C "$RUNTIME_DIR" --strip-components=1
rm python.tar.gz

echo "Installing dependencies"
cd src-tauri
./python-runtime/bin/python3 -m ensurepip --default-pip || true
./python-runtime/bin/python3 -m pip install --upgrade pip
if [ -f "../requirements.txt" ]; then
    ./python-runtime/bin/python3 -m pip install -r ../requirements.txt
else
    echo "[ERROR] requirements.txt not found. No dependencies installed"
fi

cd ..

echo "==> Finished"
