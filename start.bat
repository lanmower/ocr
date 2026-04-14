@echo off
setlocal

set SCRIPT_DIR=%~dp0
set OLLAMA_MODELS=%SCRIPT_DIR%models
set OLLAMA_HOST=127.0.0.1:11434
set PATH=%SCRIPT_DIR%ollama;%PATH%

if not exist "%OLLAMA_MODELS%" mkdir "%OLLAMA_MODELS%"

REM Start ollama serve in background
start /b "" "%SCRIPT_DIR%ollama\ollama.exe" serve

REM Wait for it to be ready
:wait
timeout /t 1 /nobreak >nul
"%SCRIPT_DIR%ollama\ollama.exe" list >nul 2>&1
if errorlevel 1 goto wait

REM Pull model if not present
"%SCRIPT_DIR%ollama\ollama.exe" list | findstr "gemma4:e2b" >nul 2>&1
if errorlevel 1 (
    echo Pulling gemma4:e2b...
    "%SCRIPT_DIR%ollama\ollama.exe" pull gemma4:e2b
)

REM Run pipeline - pass all args through
"%SCRIPT_DIR%target\release\ocr.exe" %*
