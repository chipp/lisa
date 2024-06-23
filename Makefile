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
		--label "org.opencontainers.image.source=https://github.com/chipp/lisa" \
		--cache-from=type=registry,ref=${IMAGE_ID}:cache

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
		--load \
		--label "org.opencontainers.image.source=https://github.com/chipp/lisa" \
		--cache-from=type=registry,ref=${IMAGE_ID}:cache

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

run_elisa: RUST_LOG = elisa=debug,info
run_elisa: MQTT_ADDRESS = mqtt://localhost:1883
run_elisa: MQTT_USER = elisa
run_elisa: MQTT_PASS = 123mqtt
run_elisa: VACUUM_IP = 10.0.1.150
run_elisa: VACUUM_TOKEN = $(shell op read "op://private/vacuum/credential" -n)
run_elisa:
	@RUST_LOG=${RUST_LOG} VACUUM_IP=${VACUUM_IP} VACUUM_TOKEN=${VACUUM_TOKEN} \
	MQTT_ADDRESS=${MQTT_ADDRESS} MQTT_USER=${MQTT_USER} MQTT_PASS=${MQTT_PASS} \
	cargo run --bin elisa

build_elisa: IMAGE_ID = ghcr.io/chipp/elisa
build_elisa:
	docker build . \
		--file bin/elisa/Dockerfile \
		--tag ${IMAGE_ID}:test \
		--build-arg RUST_VERSION="${RUST_VERSION}" \
		--load \
		--cache-from=type=registry,ref=${IMAGE_ID}:cache

	docker run --rm -v "${PWD}/build:/build" \
		${IMAGE_ID}:test \
		cp /root/elisa /build/elisa

run_isabel: RUST_LOG = isabel=debug,info
run_isabel: MQTT_ADDRESS = mqtt://localhost:1883
run_isabel: MQTT_USER = isabel
run_isabel: MQTT_PASS = 123mqtt
run_isabel:
	@RUST_LOG=${RUST_LOG} \
	MQTT_ADDRESS=${MQTT_ADDRESS} MQTT_USER=${MQTT_USER} MQTT_PASS=${MQTT_PASS} \
	cargo run --bin isabel

build_isabel: IMAGE_ID = ghcr.io/chipp/isabel
build_isabel:
	docker build . \
		--file bin/isabel/Dockerfile \
		--tag ${IMAGE_ID}:test \
		--build-arg RUST_VERSION="${RUST_VERSION}" \
		--load \
		--cache-from=type=registry,ref=${IMAGE_ID}:cache

	docker run --rm -v "${PWD}/build:/build" \
		${IMAGE_ID}:test \
		cp /root/isabel /build/isabel

test_isabel_libs_amd64: IMAGE_ID = ghcr.io/chipp/isabel
test_isabel_libs_amd64:
	docker build . \
		--file bin/isabel/test_libs/amd64.Dockerfile \
		--tag ${IMAGE_ID}:test_libs_amd64 \
		--build-arg RUST_VERSION="${RUST_VERSION}" \
		--load

test_isabel_libs_arm64: IMAGE_ID = ghcr.io/chipp/isabel
test_isabel_libs_arm64:
	docker build . \
		--file bin/isabel/test_libs/arm64.Dockerfile \
		--tag ${IMAGE_ID}:test_libs_arm64 \
		--build-arg RUST_VERSION="${RUST_VERSION}" \
		--load

run_elisheba: RUST_LOG = elisheba=debug,sonoff=debug,info
run_elisheba: KEYS = $(shell op read "op://private/elisheba devices/notesPlain")
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
		--load \
		--cache-from=type=registry,ref=${IMAGE_ID}:cache

	docker run --rm -v "${PWD}/build:/build" \
		${IMAGE_ID}:test \
		cp /root/elisheba /build/elisheba
