ARG DOCKER_REGISTRY
ARG DOCKER_ALPINE_VERSION=latest
FROM ${DOCKER_REGISTRY}alpine:${DOCKER_ALPINE_VERSION} as builder
# Downloads restcommander to /bin/restcommander:
ARG DOWNLOAD_URL=https://github.com/pouriya/RestCommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-musl-ubuntu-22.04
RUN echo "Running RestCommander downloader script"             \
    && export DEBUG=1                                          \
    && set -xe                                                 \
    && apk update                                              \
    && apk --no-cache add curl                                 \
    && curl -fsSLv --output /bin/restcommander ${DOWNLOAD_URL} \
    && chmod a+x /bin/restcommander
# Creates configuration at /restcommander:
RUN echo "Running RestCommander configuration script"         \
    && set -xe                                                \
    && mkdir -p /restcommander && cd /restcommander           \
    && mkdir -p scripts                                       \
    && mkdir -p www                                           \
    && restcommander sample config > config.toml              \
    && sed -i 's|= \"127.0.0.1\"|= \"0.0.0.0\"|g' config.toml \
    && restcommander sample self-signed-cert > cert.pem       \
    && restcommander sample self-signed-key > key.pem         \
    && restcommander sha512 admin > password-file.sha512      \
    && touch captcha.txt

FROM ${DOCKER_REGISTRY}alpine:${DOCKER_ALPINE_VERSION}
ARG RESTCOMMANDER_VERSION
LABEL "org.opencontainers.image.authors"="pouriya.jahanbakhsh@gmail.com"
LABEL "org.opencontainers.image.title"="restcommander"
LABEL "org.opencontainers.image.description"="HTTP REST API layer on top of scripts with a simple web dashboard"
LABEL "org.opencontainers.image.url"="https://github.com/pouriya/restcommander"
LABEL "org.opencontainers.image.source"="https://github.com/pouriya/restcommander"
LABEL "org.opencontainers.image.version"="${RESTCOMMANDER_VERSION}"
LABEL "org.opencontainers.image.licenses"="BSD-3-Clause"
COPY --from=builder /restcommander /restcommander
COPY --from=builder /bin/restcommander /bin/restcommander
WORKDIR /restcommander
VOLUME ["/restcommander", "/restcommander/scripts", "/restcommander/www"]
EXPOSE 1995
ENTRYPOINT ["/bin/restcommander"]
CMD ["config", "config.toml"]
