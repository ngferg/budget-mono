
.PHONY: clean-dbs
clean-dbs:
	rm -rf budget-lib/dbs/*.db

.PHONY: run-rest-fresh
run-rest-fresh:
	make clean-dbs
	SQLITE_DB_PATH="budget-lib/dbs" cargo run --bin budget-rest

.PHONY: run-rest
run-rest:
	SQLITE_DB_PATH="budget-lib/dbs" cargo run --bin budget-rest

.PHONY: run-dev-webapp
run-dev-webapp:
	npm run dev --prefix ./budget-web-app/


.PHONY: run-auth
run-auth:
	cargo run --bin auth-svc
