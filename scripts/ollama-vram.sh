#!/bin/bash
# Ollama VRAM Management Helper
# Quick control of model loading/unloading

case "$1" in
    status)
        echo "=== VRAM Usage ==="
        nvidia-smi --query-gpu=memory.used,memory.total --format=csv,noheader
        echo ""
        echo "=== Loaded Models ==="
        ollama ps
        ;;

    unload)
        echo "Unloading all models from VRAM..."
        MODELS=$(ollama ps --format json 2>/dev/null | grep -o '"name":"[^"]*"' | cut -d'"' -f4)
        if [ -z "$MODELS" ]; then
            echo "No models currently loaded"
        else
            for model in $MODELS; do
                echo "Stopping $model..."
                ollama stop "$model"
            done
            echo "VRAM freed"
        fi
        ;;

    load)
        echo "Pre-loading qwen2.5:7b into VRAM..."
        echo "test" | ollama run qwen2.5:7b >/dev/null 2>&1
        echo "Model loaded and ready"
        ollama ps
        ;;

    watch)
        echo "Watching VRAM usage (Ctrl-C to stop)..."
        watch -n 2 'nvidia-smi --query-gpu=memory.used,memory.total --format=csv && echo "" && ollama ps'
        ;;

    *)
        echo "Ollama VRAM Management"
        echo ""
        echo "Usage: $0 {status|unload|load|watch}"
        echo ""
        echo "Commands:"
        echo "  status  - Show current VRAM usage and loaded models"
        echo "  unload  - Immediately unload all models from VRAM"
        echo "  load    - Pre-load qwen2.5:7b into VRAM"
        echo "  watch   - Real-time VRAM monitoring"
        echo ""
        echo "Examples:"
        echo "  $0 status         # Check what's loaded"
        echo "  $0 unload         # Free VRAM before GPU work"
        echo "  $0 load           # Warm up model before batch jobs"
        exit 1
        ;;
esac
