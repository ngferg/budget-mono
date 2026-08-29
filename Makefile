
# Local dev only: load variables from .env if it exists (git-ignored). Recipes
# below prefix their command with $(LOAD_ENV) so a plain `KEY=value` or
# `export KEY=value` line in .env reaches the process. Production is untouched:
# the VPS manages its own environment and the prod targets never source .env.
DOTENV ?= .env
LOAD_ENV = set -a; [ -f $(DOTENV) ] && . ./$(DOTENV) || true; set +a

# SQLITE_DB_PATH default, applied only when .env (or the caller) didn't set it.
DB_PATH_DEFAULT = : $${SQLITE_DB_PATH:=budget-lib/dbs}; export SQLITE_DB_PATH

.PHONY: clean-dbs
clean-dbs:
	rm -rf budget-lib/dbs/*.db

.PHONY: migrate-dbs
migrate-dbs:
	$(LOAD_ENV); $(DB_PATH_DEFAULT); ./budget-lib/dbs/migrations/apply.sh

.PHONY: run-rest-fresh
run-rest-fresh:
	make clean-dbs
	$(LOAD_ENV); $(DB_PATH_DEFAULT); cargo run --bin budget-rest

.PHONY: run-rest
run-rest:
	$(LOAD_ENV); $(DB_PATH_DEFAULT); cargo run --bin budget-rest

# One-off: create the "Basic subscription" product + $5/mo price in Stripe.
# Needs STRIPE_SECRET_KEY (from .env or the environment); prints the
# STRIPE_PRICE_ID to set.
.PHONY: create-stripe-product
create-stripe-product:
	$(LOAD_ENV); cargo run --bin create_subscription_product

.PHONY: run-dev-webapp
run-dev-webapp:
	npm run dev --prefix ./budget-web-app/


.PHONY: run-auth
run-auth:
	$(LOAD_ENV); $(DB_PATH_DEFAULT); cargo run --bin auth-svc

.PHONY: run-prod-webapp
run-prod-webapp:
	git pull
	npm install --prefix ./budget-web-app
	npm run build --prefix ./budget-web-app
	sudo cp -r ./budget-web-app/dist/ /var/www/
	sudo systemctl restart nginx

.PHONY: build-prod-binaries
build-prod-binaries:
	git pull
	cargo build --release --bin auth-svc
	sudo systemctl restart bauth.service
	cargo build --release --bin budget-rest
	sudo systemctl restart bapi.service
