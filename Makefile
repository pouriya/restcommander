TARGET=$(shell rustc -vV | awk '$$1 == "host:"{print $$2}')
BUILD_DIR=$(CURDIR)/build
RELEASE_FILENAME_POSTFIX=
DEV_CMD=${BUILD_DIR}/restcommander-${VERSION}-${TARGET}-dev${RELEASE_FILENAME_POSTFIX}
RELEASE_CMD=${BUILD_DIR}/restcommander-${VERSION}-${TARGET}${RELEASE_FILENAME_POSTFIX}
DEV_DIR=$(CURDIR)/_build
BOOTSTRAP_VERSION=$(shell cat www/bootstrap-version.txt)
VERSION=$(shell cat Cargo.toml | awk 'BEGIN{FS="[ \"]"}$$1 == "application_version"{print $$4;exit}')
DOCKER_REGISTRY=
DOCKER_ALPINE_VERSION=latest
DOCKER_IMAGE_VERSION=${VERSION}


all: release

release: download-bootstrap
	@ rm -rf ${BUILD_DIR}/restcommander-* || true
	cargo build --release --target ${TARGET}
	@ mkdir -p ${BUILD_DIR} && cp ./target/${TARGET}/release/restcommander ${RELEASE_CMD}
	@ ls -sh ${BUILD_DIR}/restcommander*

deb:
	@ rm -rf ${BUILD_DIR}/restcommander-*.deb || true
	cargo deb --target ${TARGET}
	@ mkdir -p ${BUILD_DIR} && cp ./target/${TARGET}/debian/*.deb ${BUILD_DIR}/restcommander-${VERSION}-${TARGET}${RELEASE_FILENAME_POSTFIX}.deb

docker:
	docker build --build-arg DOCKER_REGISTRY=${DOCKER_REGISTRY} --build-arg DOCKER_ALPINE_VERSION=${DOCKER_ALPINE_VERSION} --build-arg RESTCOMMANDER_VERSION=${VERSION} --force-rm -t restcommander:${DOCKER_IMAGE_VERSION} -t restcommander:latest .

dev: download-bootstrap
	rm -rf ${BUILD_DIR}/restcommander-*-dev* || true
	cargo build --target ${TARGET}
	@ mkdir -p ${BUILD_DIR} && cp ./target/${TARGET}/debug/restcommander ${DEV_CMD}
	@ ls -sh ${BUILD_DIR}/restcommander-*-dev*

setup-dev: dev
	@ rm -rf ${DEV_DIR}/bin/restcommander ${DEV_DIR}/etc/restcommander/config.toml
	@ ./tools/setup.sh ${DEV_CMD} ${DEV_DIR} ${DEV_DIR}
	@ sed -i -E "s|host = (.*)|host = \"0.0.0.0\"|g" ${DEV_DIR}/etc/restcommander/config.toml
	@ sed -i -E "s|level_name = (.*)|level_name = \"debug\"|g" ${DEV_DIR}/etc/restcommander/config.toml
	@ sed -i -E "s|report = (.*)|report = \"${DEV_DIR}/var/log/restcommander/report.log\"|g" ${DEV_DIR}/etc/restcommander/config.toml

start-dev: setup-dev
	@echo ""
	@echo "Starting RestCommander"
	${DEV_DIR}/bin/restcommander config ${DEV_DIR}/etc/restcommander/config.toml

download-bootstrap: www/bootstrap.bundle.min.js www/bootstrap.min.css

www/bootstrap.bundle.min.js:
	curl --silent --output www/bootstrap.bundle.min.js https://cdn.jsdelivr.net/npm/bootstrap@${BOOTSTRAP_VERSION}/dist/js/bootstrap.bundle.min.js

www/bootstrap.min.css:
	curl --silent --output www/bootstrap.min.css https://cdn.jsdelivr.net/npm/bootstrap@${BOOTSTRAP_VERSION}/dist/css/bootstrap.min.css

exit-code-status-code-mapping:
	./tools/exit-code-status-code-mapping

clean:
	@cargo clean

clean-dev:
	@rm -rf ${DEV_DIR}

dist-clean: clean clean-dev
	@rm -rf www/bootstrap.*.js www/bootstrap.*.css

update-self-signed-certificate:
	openssl genrsa 2048 > samples/self-signed-key.pem
	echo '\n\n\n\n\n\n\n\n\n\n\n\n\n\n' | openssl req -new -x509 -nodes -days 3650 -key samples/self-signed-key.pem -out samples/self-signed-cert.pem

lint:
	cargo fmt --verbose --check
	cargo check --target ${TARGET}
#	cargo clippy --no-deps

test:
	cargo test --target ${TARGET}

.PHONY: all release deb docker dev setup-dev start-dev exit-code-status-code-mapping clean dist-clean update-self-signed-certificate lint test clean-dev
