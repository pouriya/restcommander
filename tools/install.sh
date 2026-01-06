#!/usr/bin/env sh

set -e

if [ "${DEBUG}" = "1" ]
then
  set -xe
fi

VERSION="latest"

OS_TYPE="$(uname -s)"
LINUX_CLIB_TYPE="musl"
if [ "${OS_TYPE}" = "Linux" ]
then
  if [ "${DOWNLOAD_MUSL_VERSION}" = "1" ]
  then
    LINUX_CLIB_TYPE="musl"
  fi
  # TODO: Check GLIBC version on Linux and set ${OS_TYPE} to "gnu" if its version > 2.29
#  if ldd --version 2>&1 | ...;
#  then
#    LINUX_CLIB_TYPE="gnu"
#  fi
fi

case "${OS_TYPE}" in
  Linux)
    # TODO: Check OS version and set below to 22 if possible:
    OS_TYPE=unknown-linux-${LINUX_CLIB_TYPE}-ubuntu-20.04
    ;;
  Darwin)
    # TODO: Check OS version and set below to 12 if possible:
    OS_TYPE=apple-darwin-macos-11
    ;;
  MINGW* | MSYS* | CYGWIN* | Windows_NT)
    # TODO: Check OS version and set below to 2022 if possible:
    OS_TYPE=pc-windows-gnu-windows-2019.exe
    ;;
  *)
    err "Unrecognized OS type: ${OS_TYPE}"
    ;;
esac

echo "Detected that mcpd (${OS_TYPE}) would work on your system"
EXECUTABLE="mcpd-${OS_TYPE}"
DOWNLOAD_URL="https://github.com/pouriya/mcpd/releases/download/${VERSION}/mcpd-${VERSION}-x86_64-${OS_TYPE}"
echo "Attempt to download mcpd from ${DOWNLOAD_URL}"
wget -O ${EXECUTABLE} ${DOWNLOAD_URL} || curl curl -SsfL -o ${EXECUTABLE} ${DOWNLOAD_URL}
chmod a+x ${EXECUTABLE}  || true
NAME_AND_VERSION="$(./${EXECUTABLE} --version)"
echo "${NAME_AND_VERSION} is downloaded to ${EXECUTABLE}"
echo "For simplicity you can rename it to just mcpd by running:"
echo "    mv ${EXECUTABLE} mcpd"
echo "Installed successfully"
