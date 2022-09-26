#!/usr/bin/env sh

echo "Running RestCommander downloader script"
export DEBUG=1
set -xe

#apk update
echo http://repository.fit.cvut.cz/mirrors/alpine/v3.8/main > /etc/apk/repositories
echo http://repository.fit.cvut.cz/mirrors/alpine/v3.8/community >> /etc/apk/repositories
apk --no-cache add curl
curl -fsSLv --output /bin/restcommander https://github.com/pouriya/RestCommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-musl-ubuntu-22.04
chmod a+x /bin/restcommander
