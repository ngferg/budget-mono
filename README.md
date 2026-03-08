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
Rust axum REST service interface for budget-lib

### budget-web-app
Vite Vue web app. Interfaces with budget-rest and auth-svc to deliver the full experience
