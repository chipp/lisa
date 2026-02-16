RUST_VERSION = $(shell cat .rust-version)

run_alisa: RUST_LOG = alisa=debug,info
run_alisa: JWT_SECRET = 123456
run_alisa: LISA_USER = chipp
run_alisa: LISA_PASSWORD = kek
run_alisa: ALICE_SKILL_ID = $(shell op read "op://private/yandex.alisa/username" -n)
run_alisa: ALICE_TOKEN = $(shell op read "op://private/yandex.alisa/credential" -n)
run_alisa: MQTT_ADDRESS = mqtt://localhost:1883
run_alisa: MQTT_USER = alisa
run_alisa: MQTT_PASS = 123mqtt
run_alisa:
	@RUST_LOG=${RUST_LOG} JWT_SECRET=${JWT_SECRET} \
	LISA_USER=${LISA_USER} LISA_PASSWORD=${LISA_PASSWORD} \
	ALICE_SKILL_ID=${ALICE_SKILL_ID} ALICE_TOKEN=${ALICE_TOKEN} \
	MQTT_ADDRESS=${MQTT_ADDRESS} MQTT_USER=${MQTT_USER} MQTT_PASS=${MQTT_PASS} \
	cargo run --bin alisa

build_alisa: IMAGE_ID = ghcr.io/chipp/alisa
build_alisa:
	docker build . \
		--file bin/alisa/Dockerfile \
		--tag ${IMAGE_ID}:test \
		--build-arg RUST_VERSION="${RUST_VERSION}" \
		--load \
		--label "org.opencontainers.image.source=https://github.com/chipp/lisa"

	docker run --rm -v "${PWD}/build:/build" \
		${IMAGE_ID}:test \
		cp /root/alisa /build/alisa

	docker build . \
		--file conf/arm64.Dockerfile \
		--load \
		--platform linux/arm64 \
		--progress plain \
		--tag ${IMAGE_ID}:latest \
		--build-arg BINARY=alisa \
		--label "org.opencontainers.image.source=https://github.com/chipp/lisa"

run_elizabeth: RUST_LOG = elizabeth=debug,info
run_elizabeth: MQTT_ADDRESS = mqtt://localhost:1883
run_elizabeth: MQTT_USER = elizabeth
run_elizabeth: MQTT_PASS = 123mqtt
run_elizabeth: INSPINIA_CLIENT_ID = $(shell op read "op://private/inspinia test/username" -n)
run_elizabeth: INSPINIA_TOKEN = $(shell op read "op://private/inspinia test/credential" -n)
run_elizabeth: INSPINIA_LOGS_PATH = ${PWD}/logs
run_elizabeth:
	@RUST_LOG=${RUST_LOG} INSPINIA_LOGS_PATH=${INSPINIA_LOGS_PATH} \
	MQTT_ADDRESS=${MQTT_ADDRESS} MQTT_USER=${MQTT_USER} MQTT_PASS=${MQTT_PASS} \
	INSPINIA_CLIENT_ID=${INSPINIA_CLIENT_ID} INSPINIA_TOKEN=${INSPINIA_TOKEN} \
	cargo run --bin elizabeth

build_elizabeth: IMAGE_ID = ghcr.io/chipp/elizabeth
build_elizabeth:
	docker build . \
		--file bin/elizabeth/Dockerfile \
		--tag ${IMAGE_ID}:test \
		--build-arg RUST_VERSION="${RUST_VERSION}" \
		--load

	docker run --rm -v "${PWD}/build:/build" \
		${IMAGE_ID}:test \
		cp /root/elizabeth /build/elizabeth

	docker build . \
		--file conf/arm64.Dockerfile \
		--load \
		--platform linux/arm64 \
		--progress plain \
		--tag ${IMAGE_ID}:latest \
		--build-arg BINARY=elizabeth \
		--label "org.opencontainers.image.source=https://github.com/chipp/lisa"

run_elisa: RUST_LOG = elisa=debug,roborock=debug,info
run_elisa: MQTT_ADDRESS = mqtt://localhost:1883
run_elisa: MQTT_USER = elisa
run_elisa: MQTT_PASS = 123mqtt
run_elisa: ROBOROCK_IP = 10.0.1.150
run_elisa: ROBOROCK_DUID = $(shell op read "op://private/vacuum roborock/username" -n)
run_elisa: ROBOROCK_LOCAL_KEY = $(shell op read "op://private/vacuum roborock/credential" -n)
run_elisa:
	@RUST_LOG=${RUST_LOG} ROBOROCK_IP=${ROBOROCK_IP} \
	ROBOROCK_DUID=${ROBOROCK_DUID} ROBOROCK_LOCAL_KEY=${ROBOROCK_LOCAL_KEY} \
	MQTT_ADDRESS=${MQTT_ADDRESS} MQTT_USER=${MQTT_USER} MQTT_PASS=${MQTT_PASS} \
	cargo run --bin elisa

build_elisa: IMAGE_ID = ghcr.io/chipp/elisa
build_elisa:
	docker build . \
		--file bin/elisa/Dockerfile \
		--tag ${IMAGE_ID}:test \
		--build-arg RUST_VERSION="${RUST_VERSION}" \
		--load

	docker run --rm -v "${PWD}/build:/build" \
		${IMAGE_ID}:test \
		cp /root/elisa /build/elisa

run_isabel: RUST_LOG = isabel=debug,info
run_isabel: DB_PATH = ${PWD}/target/isabel.db
run_isabel: MQTT_ADDRESS = mqtt://localhost:1883
run_isabel: MQTT_USER = isabel
run_isabel: MQTT_PASS = 123mqtt
run_isabel:
	@RUST_LOG=${RUST_LOG} DB_PATH=${DB_PATH} \
	MQTT_ADDRESS=${MQTT_ADDRESS} MQTT_USER=${MQTT_USER} MQTT_PASS=${MQTT_PASS} \
	cargo run --bin isabel

build_isabel: IMAGE_ID = ghcr.io/chipp/isabel
build_isabel:
	docker build . \
		--file bin/isabel/Dockerfile \
		--tag ${IMAGE_ID}:test \
		--build-arg RUST_VERSION="${RUST_VERSION}" \
		--load

	docker run --rm -v "${PWD}/build:/build" \
		${IMAGE_ID}:test \
		cp /root/isabel /build/isabel

run_elisheba: RUST_LOG = elisheba=debug,sonoff=debug,info
run_elisheba: KEYS = 10020750eb=$(shell op read "op://private/elisheba devices/10020750eb"),1002074ed2=$(shell op read "op://private/elisheba devices/1002074ed2")
run_elisheba: MQTT_ADDRESS = mqtt://localhost:1883
run_elisheba: MQTT_USER = elisheba
run_elisheba: MQTT_PASS = 123mqtt
run_elisheba:
	@RUST_LOG=${RUST_LOG} KEYS='${KEYS}' \
	MQTT_ADDRESS=${MQTT_ADDRESS} MQTT_USER=${MQTT_USER} MQTT_PASS=${MQTT_PASS} \
	cargo run --bin elisheba

build_elisheba: IMAGE_ID = ghcr.io/chipp/elisheba
build_elisheba:
	docker build . \
		--file bin/elisheba/Dockerfile \
		--tag ${IMAGE_ID}:test \
		--build-arg RUST_VERSION="${RUST_VERSION}" \
		--load

	docker run --rm -v "${PWD}/build:/build" \
		${IMAGE_ID}:test \
		cp /root/elisheba /build/elisheba

test: test_alisa test_elizabeth test_elisa test_isabel test_elisheba

test_alisa: IMAGE_ID = ghcr.io/chipp/alisa
test_alisa:
	docker buildx build . --file bin/alisa/test.Dockerfile \
		--output type=cacheonly \
		--tag ${IMAGE_ID}:latest \
		--build-arg RUST_VERSION="${RUST_VERSION}"

test_elizabeth: IMAGE_ID = ghcr.io/chipp/elizabeth
test_elizabeth:
	docker buildx build . --file bin/elizabeth/test.Dockerfile \
		--output type=cacheonly \
		--tag ${IMAGE_ID}:latest \
		--build-arg RUST_VERSION="${RUST_VERSION}"

test_elisa: IMAGE_ID = ghcr.io/chipp/elisa
test_elisa:
	docker buildx build . --file bin/elisa/test.Dockerfile \
		--output type=cacheonly \
		--tag ${IMAGE_ID}:latest \
		--build-arg RUST_VERSION="${RUST_VERSION}"

test_isabel: IMAGE_ID = ghcr.io/chipp/isabel
test_isabel:
	docker buildx build . --file bin/isabel/test.Dockerfile \
		--output type=cacheonly \
		--tag ${IMAGE_ID}:latest \
		--build-arg RUST_VERSION="${RUST_VERSION}"

test_elisheba: IMAGE_ID = ghcr.io/chipp/elisheba
test_elisheba:
	docker buildx build . --file bin/elisheba/test.Dockerfile \
		--output type=cacheonly \
		--tag ${IMAGE_ID}:latest \
		--build-arg RUST_VERSION="${RUST_VERSION}"
