DEV_CMD=./target/debug/restcommander
DEV_DIR=tmp/
DEV_CFG=${DEV_DIR}config.toml
BOOTSTRAP_VERSION=$(shell cat www/bootstrap-version.txt)

all: release deb
	@ ls -sh *.deb
	@ ls -sh restcommander-*

release:
	$(eval VERSION=$(shell git describe --tags))
	cargo build --release
	cp ./target/release/restcommander restcommander-${VERSION}

deb: release
	cargo deb
	cp ./target/debian/*.deb .

dev: download-bootstrap
	cargo build

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
	${DEV_CMD} sample test-script > ${DEV_DIR}/scripts/test && chmod a+x ${DEV_DIR}scripts/test
	${DEV_CMD} sample test-script-info > ${DEV_DIR}scripts/test.yml
	cp www/* ${DEV_DIR}www/ && rm -rf ${DEV_DIR}www/bootstrap-version.txt ${DEV_DIR}www/README.md

${DEV_CFG}:
	${DEV_CMD} sample config > ${DEV_CFG}.1
	awk '$$1 == "level_name" {$$3="\"debug\""}{print $$0}' ${DEV_CFG}.1 > ${DEV_CFG}.2
	awk '$$1 == "host" {$$3="\"127.0.0.1\""}{print $$0}' ${DEV_CFG}.2 > ${DEV_CFG}.1
	awk '$$1 == "password_file" {$$3="\"password-file.sha512\""}{print $$0}' ${DEV_CFG}.1 > ${DEV_CFG}.2
	awk '$$1 == "tls_cert_file" {$$3="\"cert.pem\""}{print $$0}' ${DEV_CFG}.2 > ${DEV_CFG}.1
	awk '$$1 == "tls_key_file" {$$3="\"key.pem\""}{print $$0}' ${DEV_CFG}.1 > ${DEV_CFG}.2
	awk '$$1 == "root_directory" {$$3="\"scripts\""}{print $$0}' ${DEV_CFG}.2 > ${DEV_CFG}.1
	awk '$$1 == "static_directory" {$$3="\"www\""}{print $$0}' ${DEV_CFG}.1 > ${DEV_CFG}.2
	awk '$$1 == "captcha_file" {$$3="\"captcha.txt\""}{print $$0}' ${DEV_CFG}.2 > ${DEV_CFG}
	rm -rf ${DEV_CFG}.1 ${DEV_CFG}.2
	${DEV_CMD} sha512 admin > ${DEV_DIR}password-file.sha512
	${DEV_CMD} sample self-signed-key > ${DEV_DIR}key.pem
	${DEV_CMD} sample self-signed-cert > ${DEV_DIR}cert.pem

exit-code-status-code-mapping:
	./tools/exit-code-status-code-mapping

clean:
	rm -rf ${DEV_DIR} restcommander *.deb *.tar.gz

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
	$(eval VERSION=$(shell git describe --tags))
	$(eval ARCHIVE_GENERIC_EXCLUDE=--exclude='restcommander' --exclude='*.deb' --exclude='*.tar.gz' --exclude='target' --exclude='tmp' --exclude='src/www/*')
	cd .. && tar ${ARCHIVE_GENERIC_EXCLUDE} -zcvf restcommander-${VERSION}.tar.gz RestCommander && cd RestCommander && mv ../restcommander-${VERSION}.tar.gz .
	cd .. && tar ${ARCHIVE_GENERIC_EXCLUDE} --exclude='.git' -zcvf restcommander-${VERSION}-src.tar.gz RestCommander && cd RestCommander && mv ../restcommander-${VERSION}-src.tar.gz .
	ls -sh *.tar.gz

.PHONY: all release deb dev setup-dev start-dev exit-code-status-code-mapping clean dist-clean update-self-signed-certificate lint
