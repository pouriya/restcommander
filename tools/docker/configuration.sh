#!/usr/bin/env sh

echo "Running RestCommander configuration script"
set -xe

mkdir -p /restcommander && cd /restcommander
mkdir -p scripts
mkdir -p www
restcommander sample config > config.toml
restcommander sample self-signed-cert > cert.pem
restcommander sample self-signed-key > key.pem
restcommander sha512 admin > password-file.sha512
touch captcha.txt
