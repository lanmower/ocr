# AGENTS.md — Non-obvious technical caveats

## GPU Variant Selection (setup.bat)

GPU detection uses `wmic path win32_videocontroller get name`:
- NVIDIA in name → CUDA variant (`llama-b8785-bin-win-cuda-12.4-x64.zip` + `cudart-llama-bin-win-cuda-12.4-x64.zip`)
- AMD/Radeon/Intel in name → Vulkan variant (`llama-b8785-bin-win-vulkan-x64.zip`)
- No match → CPU variant (`llama-b8785-bin-win-cpu-x64.zip`)

Selected variant is written to `tmp-llama/variant.txt`. `start.bat` reads it and passes `-ngl 99` (GPU) or `-ngl 0` (CPU) to llama-server.

The CUDA variant requires **two** ZIPs: the main binary ZIP and the cudart ZIP alongside it.

## Release ZIPs (GH Actions)

`release.yml` produces three artifacts: `ocr-cpu.zip`, `ocr-cuda.zip`, `ocr-vulkan.zip`. Each bundles `ocr.exe` + matching `llama-server` + `web/` + `*.bat`.
