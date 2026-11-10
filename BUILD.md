This project is build on top of [Tauri Framework](https://tauri.app/) so follow its instructions on how to build a basic Tauri then follow this guide.

Windows
---
1. Clone an embedded Python (3.14.9) inside a folder named "python-runtime"
```bash
Invoke-WebRequest -Uri "https://www.python.org/ftp/python/3.14.9/python-3.14.9-embed-amd64.zip" -OutFile "python.zip"
Expand-Archive -Path .\python.zip -DestinationPath .\src-tauri\python-runtime
rm python.zip
```
2. Descomente 'import site' dentro do arquivo "python314._pth"  
3. Instale o Pip manager
   ```bash
   Invoke-WebRequest -Uri "https://bootstrap.pypa.io/get-pip.py" -OutFile "get-pip.py"
   .\src-tauri\python-runtime\python.exe get-pip.py --no-warn-script-location 
   rm get-pip.py
   ```
4. Instale as bibliotecas dentro do arquivo [requirements.txt](requirements.txt) usando o executavel "python.exe"
   ```bash
   .\src-tauri\python-runtime\python.exe -m pip install --no-warn-script-location -r requirements.txt
   ```
5. Instale o [pnpm](https://pnpm.io/installation) para sua plataforma, e por fim execute esse comando.
   ```bash
   pnpm install
   pnpm tauri build
   ```

Linux
---

````bash
curl -L https://github.com/astral-sh/python-build-standalone/releases/download/20261021/cpython-3.14.9+20261021-x86_64-unknown-linux-gnu-install_only.tar.gz -o python.tar.gz

tar -xzf python.tar.gz -C src-tauri/python-runtime/ --strip-components=1
rm python.tar.gz

cd src-tauri
./python-runtime/bin/python3 -m ensurepip --default-pip || true
./python-runtime/bin/python3 -m pip install --upgrade pip
./python-runtime/bin/python3 -m pip install --no-warn-script-location -r ../requirements.txt
cd ..
````

Development
---
If you are using pnpm tauri dev, it'll use the where command, just create a venv, install the requirements, and activate it before running the command should work for both OS.

Itll use the first line that it gets from this command, so make sure its the correct before.