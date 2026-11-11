# scripts/setup-python.ps1
$ErrorActionPreference = "Stop"

Write-Host "==> Settings Standalone Python for Windows..."

$runtimeDir = "src-tauri/python-runtime"
if (-not (Test-Path $runtimeDir)) {
   New-Item -ItemType Directory -Force -Path $runtimeDir | Out-Null
}

Write-Host "Downloading Python"
$zipPath = "python.zip"
Invoke-WebRequest -Uri "https://www.python.org/ftp/python/3.14.9/python-3.14.9-embed-amd64.zip" -OutFile $zipPath
Expand-Archive -Path $zipPath -DestinationPath $runtimeDir -Force
Remove-Item $zipPath

$pthFile = Join-Path $runtimeDir "python314._pth"
if (Test-Path $pthFile) {
   Write-Host "Fixing ._pth file"
   $content = Get-Content $pthFile
   $content -replace '#import site', 'import site' | Set-Content $pthFile
}

Write-Host "Installing PIP"
$getPipPath = "get-pip.py"
Invoke-WebRequest -Uri "https://bootstrap.pypa.io/get-pip.py" -OutFile $getPipPath
$pythonExe = Join-Path $runtimeDir "python.exe"
& $pythonExe $getPipPath --no-warn-script-location
Remove-Item $getPipPath

if (Test-Path "requirements.txt") {
   Write-Host "Installing python dependencies'"
   & $pythonExe -m pip install --no-warn-script-location -r requirements.txt
}

Write-Host "==> Finished"
