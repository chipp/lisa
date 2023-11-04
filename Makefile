run_alisa: RUST_LOG = alisa=debug
run_alisa: JWT_SECRET = 123456
run_alisa: LISA_USER = chipp
run_alisa: LISA_PASSWORD = kek
run_alisa: ALICE_SKILL_ID = $(shell op read "op://private/yandex.alisa/username" | tr -d '\n')
run_alisa: ALICE_TOKEN = $(shell op read "op://private/yandex.alisa/credential" | tr -d '\n')
run_alisa: MQTT_ADDRESS = mqtt://localhost:1883
run_alisa:
	@RUST_LOG=${RUST_LOG} JWT_SECRET=${JWT_SECRET} \
	LISA_USER=${LISA_USER} LISA_PASSWORD=${LISA_PASSWORD} \
	ALICE_SKILL_ID=${ALICE_SKILL_ID} ALICE_TOKEN=${ALICE_TOKEN} \
	MQTT_ADDRESS=${MQTT_ADDRESS} \
	cargo run --bin alisa

run_elizabeth: RUST_LOG = elizabeth=debug
run_elizabeth: MQTT_ADDRESS = mqtt://localhost:1883
run_elizabeth: INSPINIA_TOKEN = $(shell op read "op://private/inspinia/credential" | tr -d '\n')
run_elizabeth:
	@RUST_LOG=${RUST_LOG} MQTT_ADDRESS=${MQTT_ADDRESS} \
	INSPINIA_TOKEN=${INSPINIA_TOKEN} \
	cargo run --bin elizabeth
