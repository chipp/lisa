.PHONY: lisa isabel

LISA_ID="ghcr.io/chipp/lisa:latest"
ISABEL_ID="ghcr.io/chipp/isabel:latest"

lisa:
	docker build . -f Dockerfile.lisa -t $(LISA_ID)

push: lisa
	docker push $(LISA_ID)

deploy: lisa
	docker image save -o lisa.tar $(LISA_ID)
	scp Makefile docker-compose.yml lisa.tar ezio:web/lisa
	ssh ezio "cd web/lisa; make install; docker-compose logs -f"

logs:
	ssh ezio "cd web/lisa; docker-compose logs -f"

install:
# 	docker pull $(LISA_ID)
	docker-compose down || true
	docker image rm $(LISA_ID)
	docker image load -i lisa.tar
	docker-compose up -d

isabel:
	docker build . -f Dockerfile.isabel -t $(ISABEL_ID) -o build
	scp build/root/isabel pi:
