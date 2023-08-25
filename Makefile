LISA_ID="ghcr.io/chipp/lisa"
ISABEL_ID="ghcr.io/chipp/isabel"


lisa:
	docker build . -f bin/lisa/Dockerfile -t $(LISA_ID):latest \
		--cache-from=$(LISA_ID):cache

deploy_lisa: lisa
	docker image save -o lisa.tar $(LISA_ID)
	scp Makefile bin/lisa/docker-compose.yml lisa.tar ezio:web/lisa_prod
	ssh ezio "cd web/lisa_prod; make install_lisa_from_tar; docker-compose logs -f"

logs_lisa:
	ssh ezio "cd web/lisa; docker-compose logs -f"

install_lisa_from_tar:
	docker-compose down || true
	docker image rm $(LISA_ID) || true
	docker image load -i lisa.tar
	docker-compose up -d

install_lisa:
	docker-compose down || true
	docker image rm $(LISA_ID)
	docker pull $(LISA_ID)
	docker-compose up -d

run_alisa: RUST_LOG = info
run_alisa: JWT_SECRET = 123456
run_alisa: LISA_USER = chipp
run_alisa: LISA_PASSWORD = kek
run_alisa: ALICE_SKILL_ID = invalid
run_alisa: ALICE_TOKEN = super_invalid
run_alisa: MQTT_ADDRESS = mqtt://localhost:1883
run_alisa:
	RUST_LOG=${RUST_LOG} JWT_SECRET=${JWT_SECRET} \
	LISA_USER=${LISA_USER} LISA_PASSWORD=${LISA_PASSWORD} \
	ALICE_SKILL_ID=${ALICE_SKILL_ID} ALICE_TOKEN=${ALICE_TOKEN} \
	cargo run --bin alisa



isabel:
	docker build . -f bin/isabel/Dockerfile -t $(ISABEL_ID):latest -o build \
		--cache-from=$(ISABEL_ID):cache

deploy_isabel: isabel
	ssh pi "sudo systemctl stop isabel.service"
	scp build/root/isabel pi:/usr/local/bin
	ssh pi "sudo systemctl start isabel.service"
	ssh pi "journalctl -u isabel.service -b -f"

logs_isabel:
	ssh pi "journalctl -u isabel.service -b -f"

run_isabel: RUST_LOG = trace
run_isabel: ELISHEBA_TOKEN = 0000000000000000000000000000000000000000000000000000000000000000
run_isabel: VACUUM_TOKEN = 704c666b4c373375446367447a6c5632
run_isabel:
	RUST_LOG=${RUST_LOG} ELISHEBA_TOKEN=${ELISHEBA_TOKEN} VACUUM_TOKEN=${VACUUM_TOKEN} cargo run --bin isabel



action_deploy:
	make install_lisa
	ssh pi "sudo service isabel stop"
	scp isabel pi:/usr/local/bin
	ssh pi "sudo service isabel start"

