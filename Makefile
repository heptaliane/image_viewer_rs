setup:
	docker-compose build

dev:
	docker-compose run build_env cargo run tauri dev

build:
	docker-compose run build_env cargo run tauri build
