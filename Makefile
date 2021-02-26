IMAGE_ID="ghcr.io/chipp/lisa:latest"

build:
	docker build . -t $(IMAGE_ID)

push: build
	docker push $(IMAGE_ID)

deploy: build
	docker image save -o lisa.tar $(IMAGE_ID)
	scp Makefile docker-compose.yml lisa.tar ezio:web/lisa
	ssh ezio "cd web/lisa; make install; docker-compose logs -f"

logs:
	ssh ezio "cd web/lisa; docker-compose logs -f"

install:
	docker-compose down || true
	docker image rm ghcr.io/chipp/lisa:latest
	docker image load -i lisa.tar
	docker-compose up -d

# 	docker pull ghcr.io/chipp/lisa:latest