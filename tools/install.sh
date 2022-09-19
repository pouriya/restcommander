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

_binary=restcommander-$_ostype
curl -LSsf --output $_binary https://github.com/pouriya/RestCommander/releases/download/$_version/restcommander-$_version-x86_64-$_ostype
chmod a+x $_binary || true
_version="$(./$_binary --version)"
echo $_version is downloaded to $_binary
