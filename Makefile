UID=$(shell id -u)
GID=$(shell id -g)

setup:
	docker-compose build \
		--build-arg UID=$(UID) \
		--build-arg GID=$(GID)

dev:
	docker-compose run -u $(UID):$(GID) build_env cargo run tauri dev

build:
	docker-compose run -u $(UID):$(GID) build_env cargo run tauri build
