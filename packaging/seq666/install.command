#!/bin/sh
# Installs SEQ 666 for the current user and clears macOS's quarantine flag.
cd "$(dirname "$0")" || exit 1
VST3="$HOME/Library/Audio/Plug-Ins/VST3"
CLAP="$HOME/Library/Audio/Plug-Ins/CLAP"
mkdir -p "$VST3" "$CLAP"
rm -rf "$VST3/Seq666.vst3" "$CLAP/Seq666.clap"
cp -R Seq666.vst3 "$VST3/" && cp -R Seq666.clap "$CLAP/"
xattr -dr com.apple.quarantine "$VST3/Seq666.vst3" "$CLAP/Seq666.clap" 2>/dev/null
echo "SEQ 666 instalado en $VST3 y $CLAP"
