build:
	docker build . -t docker.pkg.github.com/chipp/lisa/lisa:latest

push: build
	docker push docker.pkg.github.com/chipp/lisa/lisa:latest

install:
	docker-compose down || true
	docker pull docker.pkg.github.com/chipp/lisa/lisa:latest
	docker-compose up -d
