build:
	docker build . -t ghcr.io/chipp/lisa:latest

push: build
	docker push ghcr.io/chipp/lisa:latest

deploy:
	scp Makefile docker-compose.yml ezio:web/lisa
	ssh ezio "cd web/lisa; make install; docker-compose logs -f"

install:
	docker-compose down || true
	docker pull ghcr.io/chipp/lisa:latest
	docker-compose up -d
