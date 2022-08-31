DEV_CMD=./target/debug/restcommander
DEV_DIR=tmp/
DEV_CFG=${DEV_DIR}config.toml

all: release deb

release:
	cargo build --release
	cp ./target/release/restcommander .

deb: release
	cargo deb
	cp ./target/debian/*.deb .

dev:
	cargo build

setup-dev: dev ${DEV_DIR} ${DEV_CFG}

start-dev: setup-dev
	cd ${DEV_DIR} && ../${DEV_CMD} config config.toml

${DEV_DIR}:
	mkdir -p ${DEV_DIR}
	mkdir -p ${DEV_DIR}www
	mkdir -p ${DEV_DIR}scripts
	${DEV_CMD} sample test-script > ${DEV_DIR}/scripts/test && chmod a+x ${DEV_DIR}scripts/test
	${DEV_CMD} sample test-script-info > ${DEV_DIR}scripts/test.yml

${DEV_CFG}:
	${DEV_CMD} sample config > ${DEV_CFG}.1
	awk '$$1 == "level_name" {$$3="\"debug\""}{print $$0}' ${DEV_CFG}.1 > ${DEV_CFG}.2
	awk '$$1 == "host" {$$3="\"127.0.0.1\""}{print $$0}' ${DEV_CFG}.2 > ${DEV_CFG}.1
	awk '$$1 == "password_file" {$$3="\"password-file.sha512\""}{print $$0}' ${DEV_CFG}.1 > ${DEV_CFG}.2
	awk '$$1 == "tls_cert_file" {$$3="\"cert.pem\""}{print $$0}' ${DEV_CFG}.2 > ${DEV_CFG}.1
	awk '$$1 == "tls_key_file" {$$3="\"key.pem\""}{print $$0}' ${DEV_CFG}.1 > ${DEV_CFG}.2
	awk '$$1 == "root_directory" {$$3="\"scripts\""}{print $$0}' ${DEV_CFG}.2 > ${DEV_CFG}.1
	awk '$$1 == "static_directory" {$$3="\"www\""}{print $$0}' ${DEV_CFG}.1 > ${DEV_CFG}
	awk '$$1 == "captcha_file" {$$3="\"captcha.txt\""}{print $$0}' ${DEV_CFG}.1 > ${DEV_CFG}
	rm -rf ${DEV_CFG}.1 ${DEV_CFG}.2
	${DEV_CMD} sha512 admin > ${DEV_DIR}password-file.sha512
	${DEV_CMD} sample self-signed-key > ${DEV_DIR}key.pem
	${DEV_CMD} sample self-signed-cert > ${DEV_DIR}cert.pem

exit-code-status-code-mapping:
	./tools/exit-code-status-code-mapping

clean:
	rm -rf ${DEV_DIR}

dist-clean: clean
	cargo clean

update-self-signed-certificate:
	openssl genrsa 2048 > src/samples/key.pem
	echo '\n\n\n\n\n\n\n\n\n\n\n\n\n\n' | openssl req -new -x509 -nodes -days 3650 -key src/samples/key.pem -out src/samples/cert.pem


lint:
	cargo fmt --verbose --check
	cargo clippy --no-deps


.PHONY: all release deb dev setup-dev start-dev exit-code-status-code-mapping clean dist-clean update-self-signed-certificate lint
