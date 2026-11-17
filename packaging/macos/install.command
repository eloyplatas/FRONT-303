#!/bin/sh
# Installs Front 303 for the current user and clears macOS's quarantine flag.
cd "$(dirname "$0")" || exit 1
VST3="$HOME/Library/Audio/Plug-Ins/VST3"
CLAP="$HOME/Library/Audio/Plug-Ins/CLAP"
mkdir -p "$VST3" "$CLAP"
rm -rf "$VST3/Front303.vst3" "$CLAP/Front303.clap"
cp -R Front303.vst3 "$VST3/" && cp -R Front303.clap "$CLAP/"
xattr -dr com.apple.quarantine "$VST3/Front303.vst3" "$CLAP/Front303.clap" 2>/dev/null
echo "Front 303 instalado en $VST3 y $CLAP"
