#!/usr/bin/env sh

set -e

_version="latest"

_ostype="$(uname -s)"
# TODO: Check GLIBC version and set it to "gnu" if version > 2.29
_clibtype="musl"

if [ "$_ostype" = Linux ]; then
  if ldd --version 2>&1 | grep -q 'musl'; then
    _clibtype="musl"
  fi
fi
case "$_ostype" in
  Linux)
    _ostype=unknown-linux-$_clibtype-ubuntu-22.04
    ;;
  Darwin)
    _ostype=apple-darwin-macos-12
    ;;
  MINGW* | MSYS* | CYGWIN* | Windows_NT)
    _ostype=pc-windows-gnu-windows-2022.exe
    ;;
  *)
    err "unrecognized OS type: $_ostype"
    ;;
esac

echo "Detected that RestCommander ($_ostype) would work on your system"
_binary=restcommander-$_ostype
_download_url=https://github.com/pouriya/RestCommander/releases/download/$_version/restcommander-$_version-x86_64-$_ostype
echo "Attempt to download RestCommander from $_download_url"
curl -LSsf --output $_binary $_download_url
chmod a+x $_binary || true
_version="$(./$_binary --version)"
echo "$_version is downloaded to $_binary"
echo "For simplicity you can rename it to restcommander by running:"
echo "    mv $_binary restcommander"
echo "Installed successfully"
