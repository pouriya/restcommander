ARG DOCKER_REGISTRY
ARG DOCKER_ALPINE_VERSION=latest
FROM ${DOCKER_REGISTRY}alpine:${DOCKER_ALPINE_VERSION} as builder
# Downloads restcommander to /bin/restcommander:
COPY tools/docker/downloader.sh .
RUN chmod a+x downloader.sh && ./downloader.sh
# Creates configuration at /restcommander:
COPY tools/docker/configuration.sh .
RUN chmod a+x configuration.sh && ./configuration.sh

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
