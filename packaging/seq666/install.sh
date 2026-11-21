#!/bin/sh
# Installs SEQ 666 for the current user: the VST3 into ~/.vst3 and the CLAP into ~/.clap.
# With --uninstall it removes them again.
set -e
here="$(cd "$(dirname "$0")" && pwd)"
vst3="$HOME/.vst3"
clap="$HOME/.clap"
if [ "$1" = "--uninstall" ]; then
    rm -rf "$vst3/Seq666.vst3" "$clap/Seq666.clap"
    echo "SEQ 666 removed."
    exit 0
fi
mkdir -p "$vst3" "$clap"
rm -rf "$vst3/Seq666.vst3"
cp -r "$here/Seq666.vst3" "$vst3/"
cp "$here/Seq666.clap" "$clap/"
echo "Installed:"
echo "  $vst3/Seq666.vst3"
echo "  $clap/Seq666.clap"
echo "In REAPER: Options > Preferences > Plug-ins > VST, then 'Re-scan'."
