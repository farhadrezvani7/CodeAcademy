.PHONY: db api web test sync check content-check

export DATABASE_URL ?= postgres://codeacademy:codeacademy_dev@localhost:5435/codeacademy

db:            ## start PostgreSQL (docker, port 5435)
	docker compose up -d db

api: db        ## run the Rust API on :8080 (migrates + syncs content on start)
	cd backend && cargo run

web:           ## run the React app on :5173 (proxies /api to :8080)
	cd frontend && npm run dev

sync: db       ## apply edited backend/seed/*.json to the database (keeps student data)
	cd backend && cargo run -- --sync-only

test: db       ## all backend + frontend tests
	cd backend && cargo test
	cd frontend && npm test

check:         ## lint + typecheck
	cd backend && cargo clippy --all-targets -- -D warnings
	cd frontend && npm run typecheck && npm run lint
	python3 backend/seed/validate_lessons.py backend/seed/lessons/*.json
	python3 backend/seed/validate_dictionary.py

content-check: ## analyze every Dart example with the Dart analyzer (VERIFY=<flutter project>)
	python3 backend/seed/check_code.py $(VERIFY) backend/seed/lessons/*.json backend/seed/dictionary.json
