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

run_elizabeth: RUST_LOG = elizabeth=debug,info
run_elizabeth: MQTT_ADDRESS = mqtt://localhost:1883
run_elizabeth: MQTT_USER = elizabeth
run_elizabeth: MQTT_PASS = 123mqtt
run_elizabeth: INSPINIA_CLIENT_ID = $(shell op read "op://private/inspinia test/username" -n)
run_elizabeth: INSPINIA_TOKEN = $(shell op read "op://private/inspinia test/credential" -n)
run_elizabeth:
	@RUST_LOG=${RUST_LOG} \
	MQTT_ADDRESS=${MQTT_ADDRESS} MQTT_USER=${MQTT_USER} MQTT_PASS=${MQTT_PASS} \
	INSPINIA_CLIENT_ID=${INSPINIA_CLIENT_ID} INSPINIA_TOKEN=${INSPINIA_TOKEN} \
	cargo run --bin elizabeth

run_elisa: RUST_LOG = elisa=debug,info
run_elisa: MQTT_ADDRESS = mqtt://localhost:1883
run_elisa: MQTT_USER = elisa
run_elisa: MQTT_PASS = 123mqtt
run_elisa: VACUUM_IP = 192.168.1.150
run_elisa: VACUUM_TOKEN = $(shell op read "op://private/vacuum/credential" -n)
run_elisa:
	@RUST_LOG=${RUST_LOG} VACUUM_IP=${VACUUM_IP} VACUUM_TOKEN=${VACUUM_TOKEN} \
	MQTT_ADDRESS=${MQTT_ADDRESS} MQTT_USER=${MQTT_USER} MQTT_PASS=${MQTT_PASS} \
	SSL_CERT_FILE=${SSL_CERT_FILE} SSL_CERT_DIR=${SSL_CERT_DIR} \
	cargo run --bin elisa

run_isabel: RUST_LOG = isabel=debug,bluetooth=debug
run_isabel: MQTT_ADDRESS = mqtt://localhost:1883
run_isabel: MQTT_USER = isabel
run_isabel: MQTT_PASS = 123mqtt
run_isabel:
	@RUST_LOG=${RUST_LOG} SSL_CERT_FILE=${SSL_CERT_FILE} SSL_CERT_DIR=${SSL_CERT_DIR} \
	MQTT_ADDRESS=${MQTT_ADDRESS} MQTT_USER=${MQTT_USER} MQTT_PASS=${MQTT_PASS} \
	cargo run --bin isabel
