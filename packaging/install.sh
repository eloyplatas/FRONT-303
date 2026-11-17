#!/bin/sh
# Installs Front 303 for the current user: the VST3 into ~/.vst3 and the CLAP into ~/.clap.
# With --uninstall it removes them again.
set -e
here="$(cd "$(dirname "$0")" && pwd)"
vst3="$HOME/.vst3"
clap="$HOME/.clap"
if [ "$1" = "--uninstall" ]; then
    rm -rf "$vst3/Front303.vst3" "$clap/Front303.clap"
    echo "Front 303 removed."
    exit 0
fi
mkdir -p "$vst3" "$clap"
rm -rf "$vst3/Front303.vst3"
cp -r "$here/Front303.vst3" "$vst3/"
cp "$here/Front303.clap" "$clap/"
echo "Installed:"
echo "  $vst3/Front303.vst3"
echo "  $clap/Front303.clap"
echo "In REAPER: Options > Preferences > Plug-ins > VST, then 'Re-scan'."
