set export

run:
    APP_ENV=local cargo watch -x run

run-dev:
    APP_ENV=dev cargo run

run-stage:
    APP_ENV=stage cargo run

run-prod:
    APP_ENV=prod cargo run