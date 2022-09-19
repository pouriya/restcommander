ARG DOCKER_REGISTRY
ARG ALPINE_VERSION=latest
FROM ${DOCKER_REGISTRY}alpine:${ALPINE_VERSION} as downloader
RUN apk update
RUN apk --no-cache add curl
RUN curl -fsSL --output /bin/restcommander https://github.com/pouriya/RestCommander/releases/download/latest/restcommander-latest-x86_64-unknown-linux-musl-ubuntu-22.04
RUN chmod a+x /bin/restcommander
RUN mkdir -p /restcommander && cd /restcommander && mkdir -p scripts && mkdir -p www
COPY src/samples/restcommander.toml /restcommander/config.toml
RUN sed -i 's|= \"127.0.0.1\"|= \"0.0.0.0\"|g' /restcommander/config.toml
COPY src/samples/cert.pem /restcommander/
COPY src/samples/key.pem /restcommander/
RUN restcommander sample shell > /restcommander/scripts/hello-world
RUN chmod a+x /restcommander/scripts/hello-world
RUN echo "description: |" > /restcommander/scripts/hello-world.yml
RUN echo "  This is a sample shell script that prints <text class=\"text-secondary\">\"Hello World!\"</text>." >> /restcommander/scripts/hello-world.yml
RUN echo "  You can mount your own scripts directory to <text class=\"text-secondary\">/restcommander/scripts</text> to load and run yours." >> /restcommander/scripts/hello-world.yml
RUN echo "  Also you can mount your own static directory to <text class=\"text-secondary\">/restcommander/www</text> to serve your own dashboard files." >> /restcommander/scripts/hello-world.yml
RUN echo "  To change configuration, Mount your own RestCommander <text class=\"text-secondary\">*.toml</text> file to <text class=\"text-secondary\">/restcommander/config.toml</text>." >> /restcommander/scripts/hello-world.yml
RUN echo "  Need a sample configuration? Run <text class=\"text-secondary\">docker run restcommander sample config</text> to get one." >> /restcommander/scripts/hello-world.yml
RUN restcommander sha512 admin > /restcommander/password-file.sha512
RUN touch /restcommander/captcha.txt

FROM ${DOCKER_REGISTRY}alpine:${ALPINE_VERSION}
COPY --from=downloader /restcommander /restcommander
COPY --from=downloader /bin/restcommander /bin/restcommander
WORKDIR /restcommander
VOLUME /restcommander
EXPOSE 1995
ENTRYPOINT ["/bin/restcommander"]
CMD ["config", "config.toml"]
