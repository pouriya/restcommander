ARG DOCKER_REGISTRY
ARG DOCKER_ALPINE_VERSION=latest
FROM ${DOCKER_REGISTRY}alpine:${DOCKER_ALPINE_VERSION} as builder
# Download & setup restcommander:
ARG RESTCOMMANDER_VERSION=latest
ARG DOWNLOAD_URL=https://github.com/pouriya/RestCommander/releases/download/${RESTCOMMANDER_VERSION}/install.sh

RUN set -xe && wget --no-check-certificate -O download.sh ${DOWNLOAD_URL} && chmod a+x download.sh && DEBUG=1 DOWNLOAD_MUSL_VERSION=1 ./download.sh
# To test docker build in develompent:
#     Run `make dev TARGET=x86_64-unknown-linux-musl`
#     Comment above `RUN ...`
#     Uncomment below `COPY ...`
#     Run `[sudo] make docker`
#COPY build/restcommander*musl*dev* .

RUN mv restcommander-*-dev* restcommander && ls -lash restcommander
COPY tools/setup.sh .
RUN rm -rf /install-restcommander && chmod a+x setup.sh && DEBUG=1 ./setup.sh ./restcommander /install-restcommander ""
RUN sed -i -E "s|host = (.*)|host = \"0.0.0.0\"|g" /install-restcommander/etc/restcommander/config.toml
RUN sed -i -E "s|host = (.*)|host = \"0.0.0.0\"|g" /install-restcommander/etc/restcommander/config.toml.example


FROM ${DOCKER_REGISTRY}alpine:${DOCKER_ALPINE_VERSION}
ARG RESTCOMMANDER_VERSION
LABEL "org.opencontainers.image.authors"="pouriya.jahanbakhsh@gmail.com"
LABEL "org.opencontainers.image.title"="restcommander"
LABEL "org.opencontainers.image.description"="HTTP REST API layer on top of scripts with a simple web dashboard"
LABEL "org.opencontainers.image.url"="https://github.com/pouriya/restcommander"
LABEL "org.opencontainers.image.source"="https://github.com/pouriya/restcommander"
LABEL "org.opencontainers.image.version"="${RESTCOMMANDER_VERSION}"
LABEL "org.opencontainers.image.licenses"="BSD-3-Clause"
ENV DOCKER_CONTAINER="1"
WORKDIR /
COPY --from=builder /install-restcommander /
EXPOSE 1995
ENTRYPOINT ["/bin/restcommander"]
CMD ["config", "/etc/restcommander/config.toml"]
