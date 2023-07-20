setup:
	cargo install sqlx-cli
	cargo install cargo-watch
	docker compose build

up:
	docker compose up -d

down:
	docker compose down

dev:
	sqlx db create
	sqlx migrate run
	cargo watch -x run

test:
	cargo test