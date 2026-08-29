# budget-mono
Up and running at https://febudget.com

## Projects
### auth-svc
Rust axum web service that handles e-mail auth.
Need to set SMTP_USER, SMTP_PASS, and SMTP_HOST env variables to send e-mail codes

### bruno
Contains REST requests for testing services. https://www.usebruno.com

### budget-lib
Rust library containing core logic and sqlite implementation

### budget-rest
Rust axum REST service interface for budget-lib.

Handles subscription payments through Stripe Managed Payments (Stripe is the
merchant of record and collects tax):

- `POST /users/subscription/checkout` opens a Stripe Checkout Session for the
  $5/month plan and returns its hosted URL. The web app redirects here when the
  API answers `402` (free trial lapsed).
- `POST /webhooks/stripe` consumes `checkout.session.completed`, verifies the
  signature, and flips the account to `active`, storing the Stripe
  customer/subscription ids in the user's `subscription` row.

Setup:

1. `cp .env.example .env` and fill in the `STRIPE_*` keys from the Stripe
   Dashboard.
2. `STRIPE_SECRET_KEY=sk_test_... make create-stripe-product` once, then put the
   printed `STRIPE_PRICE_ID` in the environment.
3. Locally, forward webhooks with
   `stripe listen --forward-to localhost:3000/webhooks/stripe` and use the
   `whsec_...` it prints as `STRIPE_WEBHOOK_SECRET`.

### budget-web-app
Vite Vue web app. Interfaces with budget-rest and auth-svc to deliver the full experience
