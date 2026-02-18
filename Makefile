
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

.PHONY: run-prod-webapp
run-prod-webapp:
	npm run build --prefix ./budget-web-app
	sudo cp -r ./budget-web-app/dist/ /var/www/
	sudo systemctl restart nginx

.PHONY: build-prod-binaries
build-prod-binaries:
	cargo build --release --bin auth-svc
	sudo systemctl restart bauth.service
	cargo build --release --bin budget-rest
	sudo systemctl restart bapi.service
