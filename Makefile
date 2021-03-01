.PHONY: lisa isabel

LISA_ID="ghcr.io/chipp/lisa:latest"
ISABEL_ID="ghcr.io/chipp/isabel:latest"


lisa:
	docker build . -f Dockerfile.lisa -t $(LISA_ID)

deploy_lisa: lisa
	docker image save -o lisa.tar $(LISA_ID)
	scp Makefile docker-compose.yml lisa.tar ezio:web/lisa
	ssh ezio "cd web/lisa; make install_lisa; docker-compose logs -f"

logs_lisa:
	ssh ezio "cd web/lisa; docker-compose logs -f"

install_lisa:
	docker-compose down || true
	docker image rm $(LISA_ID)
	docker image load -i lisa.tar
	docker-compose up -d

run_lisa:
	RUST_LOG=info cargo run --bin lisa


isabel:
	docker build . -f Dockerfile.isabel -t $(ISABEL_ID) -o build

deploy_isabel: isabel
	ssh pi "sudo systemctl stop isabel.service"
	scp build/root/isabel pi:/usr/local/bin
	ssh pi "sudo systemctl start isabel.service"
	ssh pi "journalctl -u isabel.service -b -f"

logs_isabel:
	ssh pi "journalctl -u isabel.service -b -f"

run_isabel:
	RUST_LOG=info cargo run --bin isabel
