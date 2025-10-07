#!/bin/bash
# zip-firmware.sh
# Zips the latest firmware build artifact for easy download from Codespaces

set -e

# Path to the build output directory (relative to workspace root for Codespaces)
BUILD_DIR="embassy/examples/rp/target/thumbv6m-none-eabi/release"
OUTPUT_DIR="firmware-artifacts"


# Name of the binary (adjust if needed)
BIN_NAME="crusty"

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Find the .uf2 and .bin files (if they exist)
UF2_FILE=$(find "$BUILD_DIR" -maxdepth 1 -name "*.uf2" | head -n 1)
BIN_FILE="$BUILD_DIR/$BIN_NAME"

ZIP_NAME="firmware-$(date +%Y%m%d-%H%M%S).zip"
ZIP_PATH="$(pwd)/$OUTPUT_DIR/$ZIP_NAME"
# Zip available artifacts

cd "$BUILD_DIR"
FILES_TO_ZIP=()
# Check for .bin file (should be in current dir as $BIN_NAME)
if [ -f "$BIN_NAME" ]; then
  FILES_TO_ZIP+=("$BIN_NAME")
fi
# Check for .uf2 file (use basename after cd)
if [ -n "$UF2_FILE" ]; then
  UF2_BASENAME="$(basename "$UF2_FILE")"
  if [ -f "$UF2_BASENAME" ]; then
    FILES_TO_ZIP+=("$UF2_BASENAME")
  fi
fi


if [ ${#FILES_TO_ZIP[@]} -eq 0 ]; then
  echo "No firmware artifacts found to zip."
  echo "Checked for: $BIN_NAME and $UF2_BASENAME in $BUILD_DIR"
  ls -l "$BUILD_DIR"
  exit 1
fi

zip -j "$ZIP_PATH" "${FILES_TO_ZIP[@]}"
echo "Zipped firmware artifacts to $ZIP_PATH"
