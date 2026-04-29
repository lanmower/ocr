@echo off
setlocal

set SCRIPT_DIR=%~dp0
set LLM_DIR=%SCRIPT_DIR%tmp-llama
set MODEL=%LLM_DIR%\model-q4km.gguf
set MMPROJ=%LLM_DIR%\mmproj-google_gemma-4-E2B-it-f16.gguf
set SERVER=%LLM_DIR%\b8785-extracted\llama-server.exe
set LLAMA_HOST=127.0.0.1:8080
set PATH=%LLM_DIR%\b8785-extracted;%PATH%

REM Auto-fetch runtime bundle on first run
if not exist "%SERVER%" call "%SCRIPT_DIR%setup.bat" || exit /b 1
if not exist "%MODEL%" call "%SCRIPT_DIR%setup.bat" || exit /b 1
if not exist "%MMPROJ%" call "%SCRIPT_DIR%setup.bat" || exit /b 1

REM Build ocr.exe if missing
if not exist "%SCRIPT_DIR%target\release\ocr.exe" (
    echo Building ocr.exe...
    cargo build --release || exit /b 1
)

REM Kill any existing llama-server
taskkill /f /im llama-server.exe >nul 2>&1

REM Start llama-server in background
start /b "" "%SERVER%" -m "%MODEL%" --mmproj "%MMPROJ%" --host 127.0.0.1 --port 8080 -ngl 0 -rea off

REM Wait for server ready
:wait
timeout /t 2 /nobreak >nul
curl -s http://%LLAMA_HOST%/health | findstr "ok" >nul 2>&1
if errorlevel 1 goto wait

REM Run pipeline
"%SCRIPT_DIR%target\release\ocr.exe" %*
set EXIT_CODE=%ERRORLEVEL%

REM Shut down server
taskkill /f /im llama-server.exe >nul 2>&1

exit /b %EXIT_CODE%
