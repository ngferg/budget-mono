# budget-mono
Up and running at https://febudget.com

## Projects
### auth-svc
Handles e-mail auth.
Need to set SMTP_USER, SMTP_PASS, and SMTP_HOST env variables to send e-mail codes

### bruno
Contains REST requests for testing services. https://www.usebruno.com

### budget-lib
Contains core logic and sqlite implementation

### budget-rest
REST service interface for budget-lib running with axum

### budget-web-app
Vite Vue web app. Interfaces with budget-rest and auth-svc to deliver the full experience
