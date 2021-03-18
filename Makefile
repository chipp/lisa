LISA_ID="ghcr.io/chipp/lisa"
ISABEL_ID="ghcr.io/chipp/isabel"


lisa:
	docker build . -f bin/lisa/Dockerfile -t $(LISA_ID):latest \
		--cache-from=$(LISA_ID):cache

deploy_lisa: lisa
	docker image save -o lisa.tar $(LISA_ID)
	scp Makefile bin/lisa/docker-compose.yml lisa.tar ezio:web/lisa
	ssh ezio "cd web/lisa; make install_lisa_from_tar; docker-compose logs -f"

logs_lisa:
	ssh ezio "cd web/lisa; docker-compose logs -f"

install_lisa_from_tar:
	docker-compose down || true
	docker image rm $(LISA_ID)
	docker image load -i lisa.tar
	docker-compose up -d

install_lisa:
	docker-compose down || true
	docker image rm $(LISA_ID)
	docker pull $(LISA_ID)
	docker-compose up -d

run_lisa:
	RUST_LOG=debug cargo run --bin lisa



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

run_isabel:
	RUST_LOG=debug cargo run --bin isabel



action_deploy:
	make install_lisa
	ssh pi "sudo service isabel stop"
	scp isabel pi:/usr/local/bin
	ssh pi "sudo service isabel start"

