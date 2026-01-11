
.PHONY: clean-dbs
clean-dbs:
	rm -rf budget-lib/dbs/*.db

.PHONY: run-rest-fresh
run-rest-fresh:
	make clean-dbs
	cd budget-rest
	SQLITE_DB_PATH="budget-lib/dbs" cargo run
	cd -

.PHONY: run-rest
run-rest:
	cd budget-rest
	SQLITE_DB_PATH="budget-lib/dbs" cargo run
	cd -

.PHONY: run-dev-webapp
run-dev-webapp:
	npm run dev --prefix ./budget-web-app/


.PHONY: run-auth
run-auth:
	cd auth-svc
	cargo run
	cd -
