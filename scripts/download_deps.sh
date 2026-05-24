#!/usr/bin/env bash
# Downloads all native binary dependencies that are not tracked in git:
#   lib/libvosk.so       — Vosk speech-recognition shared library
#   lib/piper/           — Piper TTS runtime + phonemizer libraries
#   lib/piper_model/     — English voice model for Piper

set -euo pipefail

VOSK_VERSION="0.3.45"
PIPER_VERSION="2023.11.14-2"
PIPER_VOICE="en_US-lessac-medium"

VOSK_URL="https://github.com/alphacep/vosk-api/releases/download/v${VOSK_VERSION}/vosk-linux-x86_64-${VOSK_VERSION}.zip"
PIPER_URL="https://github.com/rhasspy/piper/releases/download/${PIPER_VERSION}/piper_linux_x86_64.tar.gz"
MODEL_BASE="https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

mkdir -p "${ROOT}/lib/piper" "${ROOT}/lib/piper_model"

echo "==> Downloading Vosk library (${VOSK_VERSION})..."
TMP_VOSK="$(mktemp -d)"
trap 'rm -rf "${TMP_VOSK}"' EXIT
curl -fsSL "${VOSK_URL}" -o "${TMP_VOSK}/vosk.zip"
unzip -j "${TMP_VOSK}/vosk.zip" "*/libvosk.so" -d "${TMP_VOSK}"
cp "${TMP_VOSK}/libvosk.so" "${ROOT}/lib/libvosk.so"
echo "    -> lib/libvosk.so"

echo "==> Downloading Piper TTS runtime (${PIPER_VERSION})..."
TMP_PIPER="$(mktemp -d)"
trap 'rm -rf "${TMP_VOSK}" "${TMP_PIPER}"' EXIT
curl -fsSL "${PIPER_URL}" -o "${TMP_PIPER}/piper.tar.gz"
tar -xzf "${TMP_PIPER}/piper.tar.gz" -C "${TMP_PIPER}"
cp -r "${TMP_PIPER}/piper/." "${ROOT}/lib/piper/"
echo "    -> lib/piper/"

echo "==> Downloading Piper voice model (${PIPER_VOICE})..."
curl -fsSL "${MODEL_BASE}/${PIPER_VOICE}.onnx"      -o "${ROOT}/lib/piper_model/${PIPER_VOICE}.onnx"
curl -fsSL "${MODEL_BASE}/${PIPER_VOICE}.onnx.json" -o "${ROOT}/lib/piper_model/${PIPER_VOICE}.onnx.json"
echo "    -> lib/piper_model/${PIPER_VOICE}.onnx"
echo "    -> lib/piper_model/${PIPER_VOICE}.onnx.json"

echo "Done."
