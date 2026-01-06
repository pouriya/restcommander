ARG DOCKER_REGISTRY
ARG DOCKER_ALPINE_VERSION=latest
FROM ${DOCKER_REGISTRY}alpine:${DOCKER_ALPINE_VERSION} as builder
# Download & setup mcpd:
ARG MCPD_VERSION=latest
ARG DOWNLOAD_URL=https://github.com/pouriya/mcpd/releases/download/${MCPD_VERSION}/install.sh

RUN set -xe && wget --no-check-certificate -O download.sh ${DOWNLOAD_URL} && chmod a+x download.sh && DEBUG=1 DOWNLOAD_MUSL_VERSION=1 ./download.sh
# To test docker build in develompent:
#     Run `make dev TARGET=x86_64-unknown-linux-musl`
#     Comment above `RUN ...`
#     Uncomment below `COPY ...`
#     Run `[sudo] make docker`
#COPY build/mcpd*musl*dev* .

RUN mv mcpd-*-dev* mcpd && ls -lash mcpd
COPY tools/setup.sh .
RUN rm -rf /install-mcpd && chmod a+x setup.sh && DEBUG=1 ./setup.sh ./mcpd /install-mcpd ""
RUN sed -i -E "s|host = (.*)|host = \"0.0.0.0\"|g" /install-mcpd/etc/mcpd/config.toml
RUN sed -i -E "s|host = (.*)|host = \"0.0.0.0\"|g" /install-mcpd/etc/mcpd/config.toml.example


FROM ${DOCKER_REGISTRY}alpine:${DOCKER_ALPINE_VERSION}
ARG MCPD_VERSION
LABEL "org.opencontainers.image.authors"="pouriya.jahanbakhsh@gmail.com"
LABEL "org.opencontainers.image.title"="mcpd"
LABEL "org.opencontainers.image.description"="MCP daemon - expose scripts as MCP tools and resources"
LABEL "org.opencontainers.image.url"="https://github.com/pouriya/mcpd"
LABEL "org.opencontainers.image.source"="https://github.com/pouriya/mcpd"
LABEL "org.opencontainers.image.version"="${MCPD_VERSION}"
LABEL "org.opencontainers.image.licenses"="BSD-3-Clause"
ENV DOCKER_CONTAINER="1"
WORKDIR /
COPY --from=builder /install-mcpd /
EXPOSE 1995
ENTRYPOINT ["/bin/mcpd"]
CMD ["config", "/etc/mcpd/config.toml"]
