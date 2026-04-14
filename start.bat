@echo off
setlocal

set SCRIPT_DIR=%~dp0
set LLM_DIR=%SCRIPT_DIR%tmp-llama
set MODEL=%LLM_DIR%\model.gguf
set MMPROJ=%LLM_DIR%\mmproj-google_gemma-4-E2B-it-f16.gguf
set SERVER=%LLM_DIR%\unzipped\llama-server.exe
set LLAMA_HOST=127.0.0.1:8080
set PATH=%LLM_DIR%\unzipped;%PATH%

REM Start llama-server in background
start /b "" "%SERVER%" -m "%MODEL%" --mmproj "%MMPROJ%" --host 127.0.0.1 --port 8080 -ngl 0

REM Wait for server ready
:wait
timeout /t 2 /nobreak >nul
curl -s http://%LLAMA_HOST%/health | findstr "ok" >nul 2>&1
if errorlevel 1 goto wait

REM Run pipeline - pass all args through
"%SCRIPT_DIR%target\release\ocr.exe" %*
