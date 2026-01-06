ARG DOCKER_REGISTRY
ARG DOCKER_ALPINE_VERSION=latest
FROM ${DOCKER_REGISTRY}rust:alpine${DOCKER_ALPINE_VERSION} AS builder

RUN apk add --no-cache make curl

WORKDIR mcpd

COPY src src
COPY www www
COPY build.rs .
COPY Cargo.toml .
COPY Cargo.lock .
COPY Makefile .
RUN ls src .
RUN make release

RUN mkdir -p build/bin build/var/lib/mcpd
RUN mv build/mcpd-* build/bin/mcpd


FROM ${DOCKER_REGISTRY}alpine:${DOCKER_ALPINE_VERSION}
ARG MCPD_VERSION
LABEL "org.opencontainers.image.authors"="pouriya.jahanbakhsh@gmail.com"
LABEL "org.opencontainers.image.title"="mcpd"
LABEL "org.opencontainers.image.description"="MCP daemon - expose simple scripts as MCP tools and resources"
LABEL "org.opencontainers.image.url"="https://github.com/pouriya/mcpd"
LABEL "org.opencontainers.image.source"="https://github.com/pouriya/mcpd"
LABEL "org.opencontainers.image.version"="${MCPD_VERSION}"
LABEL "org.opencontainers.image.licenses"="BSD-3-Clause"
ENV DOCKER_CONTAINER="1"
WORKDIR /
COPY --from=builder /mcpd/build/ /
ENTRYPOINT ["/bin/mcpd"]
CMD ["--script-root-directory", "/var/lib/mcpd/", "--http-host", "0.0.0.0"]
