TARGET=$(shell rustc -vV | awk '$$1 == "host:"{print $$2}')
DEV_CMD=./target/${TARGET}/debug/restcommander
DEV_DIR=tmp/
DEV_CFG=${DEV_DIR}config.toml
BOOTSTRAP_VERSION=$(shell cat www/bootstrap-version.txt)
VERSION=$(shell cat Cargo.toml | awk 'BEGIN{FS="[ \"]"}$$1 == "version"{print $$4;exit}')
RELEASE_FILENAME_POSTFIX=
DOCKER_REGISTRY=
DOCKER_ALPINE_VERSION=latest
DOCKER_IMAGE_VERSION=latest


all: release
	@ ls -sh *.deb
	@ ls -sh restcommander-*

release: download-bootstrap
	cargo build --release --target ${TARGET}
	@ cp ./target/${TARGET}/release/restcommander restcommander-${VERSION}-${TARGET}${RELEASE_FILENAME_POSTFIX}

deb:
	cargo deb --target ${TARGET}
	@ cp ./target/${TARGET}/debian/*.deb restcommander-${VERSION}-${TARGET}${RELEASE_FILENAME_POSTFIX}.deb

docker:
	docker build --build-arg DOCKER_REGISTRY=${DOCKER_REGISTRY} --build-arg DOCKER_ALPINE_VERSION=${DOCKER_ALPINE_VERSION} --force-rm -t restcommander:${DOCKER_IMAGE_VERSION} .

dev: download-bootstrap
	cargo build --target ${TARGET}

setup-dev: dev ${DEV_DIR} ${DEV_CFG}

start-dev: setup-dev
	cd ${DEV_DIR} && ../${DEV_CMD} config config.toml

download-bootstrap: www/bootstrap.bundle.min.js www/bootstrap.min.css
	@ ls -sh www/bootstrap.*

www/bootstrap.bundle.min.js:
	curl --silent --output www/bootstrap.bundle.min.js https://cdn.jsdelivr.net/npm/bootstrap@${BOOTSTRAP_VERSION}/dist/js/bootstrap.bundle.min.js

www/bootstrap.min.css:
	curl --silent --output www/bootstrap.min.css https://cdn.jsdelivr.net/npm/bootstrap@${BOOTSTRAP_VERSION}/dist/css/bootstrap.min.css

${DEV_DIR}:
	mkdir -p ${DEV_DIR}
	mkdir -p ${DEV_DIR}www
	mkdir -p ${DEV_DIR}scripts
	mkdir -p ${DEV_DIR}scripts/tour
	${DEV_CMD} sample test-script > ${DEV_DIR}/scripts/test && chmod a+x ${DEV_DIR}scripts/test
	${DEV_CMD} sample test-script-info > ${DEV_DIR}scripts/test.yml
	cp www/* ${DEV_DIR}www/ && rm -rf ${DEV_DIR}www/bootstrap-version.txt ${DEV_DIR}www/README.md
	cp tools/tour/scripts/* ${DEV_DIR}scripts/tour/

${DEV_CFG}:
	${DEV_CMD} sample config > ${DEV_CFG}
	cat ${DEV_CFG} | awk '$$1 == "level_name" {$$3="\"debug\""}{print $$0}' > ${DEV_CFG}.tmp
	mv ${DEV_CFG}.tmp ${DEV_CFG}
	${DEV_CMD} sha512 admin > ${DEV_DIR}password-file.sha512
	${DEV_CMD} sample self-signed-key > ${DEV_DIR}key.pem
	${DEV_CMD} sample self-signed-cert > ${DEV_DIR}cert.pem

exit-code-status-code-mapping:
	./tools/exit-code-status-code-mapping

clean: clean-dev
	rm -rf restcommander-*
	mv src/www/mod.rs www_mod.rs && rm -rf src/www/* && mv www_mod.rs src/www/mod.rs

clean-dev:
	rm -rf ${DEV_DIR}

dist-clean: clean
	cargo clean
	rm -rf www/bootstrap.*.js www/bootstrap.*.css

update-self-signed-certificate:
	openssl genrsa 2048 > src/samples/key.pem
	echo '\n\n\n\n\n\n\n\n\n\n\n\n\n\n' | openssl req -new -x509 -nodes -days 3650 -key src/samples/key.pem -out src/samples/cert.pem

lint:
	cargo fmt --verbose --check
	cargo clippy --no-deps

archive:
	$(eval ARCHIVE_GENERIC_EXCLUDE=--exclude='restcommander' --exclude='*.deb' --exclude='*.tar.gz' --exclude='target' --exclude='tmp' --exclude='src/www/*')
	cd .. && tar ${ARCHIVE_GENERIC_EXCLUDE} -zcvf restcommander-${VERSION}.tar.gz RestCommander && cd RestCommander && mv ../restcommander-${VERSION}.tar.gz .
	cd .. && tar ${ARCHIVE_GENERIC_EXCLUDE} --exclude='.git' -zcvf restcommander-${VERSION}-src.tar.gz RestCommander && cd RestCommander && mv ../restcommander-${VERSION}-src.tar.gz .
	ls -sh *.tar.gz

.PHONY: all release deb docker dev setup-dev start-dev exit-code-status-code-mapping clean dist-clean update-self-signed-certificate lint archive clean-dev
