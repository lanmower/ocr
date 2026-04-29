@echo off
setlocal

set SCRIPT_DIR=%~dp0
set LLM_DIR=%SCRIPT_DIR%tmp-llama
set MODEL=%LLM_DIR%\model-q4km.gguf
set MMPROJ=%LLM_DIR%\mmproj-google_gemma-4-E2B-it-f16.gguf
set SERVER_DIR=%LLM_DIR%\b8785-extracted
set SERVER=%SERVER_DIR%\llama-server.exe
set SERVER_ZIP=%LLM_DIR%\b8785.zip

set MODEL_URL=https://huggingface.co/bartowski/google_gemma-4-E2B-it-GGUF/resolve/main/google_gemma-4-E2B-it-Q4_K_M.gguf
set MMPROJ_URL=https://huggingface.co/bartowski/google_gemma-4-E2B-it-GGUF/resolve/main/mmproj-google_gemma-4-E2B-it-f16.gguf
set SERVER_URL=https://github.com/ggml-org/llama.cpp/releases/download/b8785/llama-b8785-bin-win-cpu-x64.zip

if not exist "%LLM_DIR%" mkdir "%LLM_DIR%"

if not exist "%MODEL%" (
    echo Downloading model ^(~3.3 GB^)...
    curl -L -o "%MODEL%" "%MODEL_URL%" || goto fail
)

if not exist "%MMPROJ%" (
    echo Downloading multimodal projector ^(~940 MB^)...
    curl -L -o "%MMPROJ%" "%MMPROJ_URL%" || goto fail
)

if not exist "%SERVER%" (
    echo Downloading llama-server b8785 ^(~40 MB^)...
    curl -L -o "%SERVER_ZIP%" "%SERVER_URL%" || goto fail
    if not exist "%SERVER_DIR%" mkdir "%SERVER_DIR%"
    powershell -Command "Expand-Archive -Force '%SERVER_ZIP%' '%SERVER_DIR%'" || goto fail
    del "%SERVER_ZIP%"
)

echo Setup complete.
exit /b 0

:fail
echo Setup failed.
exit /b 1
