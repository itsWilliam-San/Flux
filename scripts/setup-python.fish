#!/usr/bin/evn fish

echo "==> Setting Standalone Python..."

set RUNTIME_DIR "src-tauri/python-runtime"
mkdir -p $RUNTIME_DIR

echo "Downloading standalone Python"
curl -L https://github.com/astral-sh/python-build-standalone/releases/download/20261021/cpython-3.14.9+20261021-x86_64-unknown-linux-gnu-install_only.tar.gz -o python.tar.gz
tar -xzf python.tar.gz -C $RUNTIME_DIR --stip-components=1
rm python.tar.gz

echo "Settings pip and dependencies"
set PYTHON_BIN "$RUNTIME_DIR/bin/python3"

if test -x $PYTHON_BIN
   $PYTHON_BIN -m ensurepip --default-pip
   $PYTHON_BIN -m pip install --upgrade pip

   if test -f "requirements.txt"
    $PYTHON_BIN -m pip install -r requirements.txt
   else
    echo "Failed to find requirements.txt file"
    exit 1
   end
else
    echo "Erro: Python runtime not found."
    exit 1
end

echo "Finished"
