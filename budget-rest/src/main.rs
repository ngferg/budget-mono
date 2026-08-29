use axum::{
    Router, http,
    routing::{delete, get, post},
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // reqwest talks to Stripe over rustls; install a crypto provider before the
    // first HTTPS call, same as the auth service does.
    let _ = rustls::crypto::ring::default_provider().install_default();

    // CORS configuration
    let cors = CorsLayer::permissive();
    // our router
    let app = Router::new()
        .route("/health", get(health))
        .route("/users", post(create_user).delete(delete_user))
        .route("/users/budget", post(find_budget))
        .route(
            "/users/budget/line_item",
            delete(delete_line_item)
                .post(add_line_item)
                .put(edit_line_item),
        )
        .route("/users/budget/clone_month", post(clone_month))
        .route("/users/budget/category", post(add_category))
        .route("/users/budget/csv", get(export_csv))
        .route("/users/subscription", get(get_subscription))
        .route(
            "/users/subscription/checkout",
            post(create_checkout_session),
        )
        .route("/users/subscription/cancel", post(cancel_subscription))
        .route("/webhooks/stripe", post(handle_stripe_webhook))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Budget service started on port 3000");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> (http::StatusCode, axum::Json<serde_json::Value>) {
    let healthy = r#"
    {
      "status": "healthy"
    }
    "#;
    let json = serde_json::from_str(healthy).unwrap_or_default();
    (http::StatusCode::OK, axum::Json(json))
}

async fn find_budget(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::GetBudgetRequest>,
) -> (http::StatusCode, axum::Json<serde_json::Value>) {
    if let Err(e) = verify_auth(headers, &req.hashed_email.as_str()).await {
        return (
            e,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        );
    }
    if let Err(e) = verify_entitlement(req.hashed_email.as_str()).await {
        return (
            e,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        );
    }
    let res = budget_lib::get_budget(req).await;
    match res {
        Ok(res) => (
            http::StatusCode::OK,
            axum::Json(serde_json::to_value(res).unwrap_or_default()),
        ),
        Err(e) => match e {
            budget_lib::types::GetBudgetError::Internal(e) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::to_value(e).unwrap_or_default()),
            ),
            budget_lib::types::GetBudgetError::UserDoesntExists() => (
                http::StatusCode::NOT_FOUND,
                axum::Json(serde_json::from_str("{}").unwrap_or_default()),
            ),
            budget_lib::types::GetBudgetError::BudgetDoesntExists() => (
                http::StatusCode::NOT_FOUND,
                axum::Json(serde_json::from_str("{}").unwrap_or_default()),
            ),
            budget_lib::types::GetBudgetError::DateError(e) => (
                http::StatusCode::BAD_REQUEST,
                axum::Json(
                    serde_json::from_str(format!("{{\"error\": \"{e}\"}}").as_str())
                        .unwrap_or_default(),
                ),
            ),
        },
    }
}

async fn create_user(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::CreateUserRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.hashed_email.as_str()).await {
        return e;
    }
    let res = budget_lib::create_user(req).await;
    match res {
        Ok(()) => http::StatusCode::CREATED,
        Err(e) => match e {
            budget_lib::types::CreateUserError::UserAlreadyExists() => http::StatusCode::CONFLICT,
            budget_lib::types::CreateUserError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}

async fn delete_user(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::DeleteUserRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.hashed_email.as_str()).await {
        return e;
    }
    let res = budget_lib::delete_user(req).await;
    match res {
        Ok(()) => http::StatusCode::NO_CONTENT,
        Err(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn delete_line_item(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::DeleteLineItemRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.hashed_email.as_str()).await {
        return e;
    }
    if let Err(e) = verify_entitlement(req.hashed_email.as_str()).await {
        return e;
    }
    let res = budget_lib::delete_line_item(req).await;
    match res {
        Ok(()) => http::StatusCode::NO_CONTENT,
        Err(e) => match e {
            budget_lib::types::DeleteLineItemError::UserDoesntExists() => {
                http::StatusCode::NOT_FOUND
            }
            budget_lib::types::DeleteLineItemError::LineItemDoesntExist() => {
                http::StatusCode::NOT_FOUND
            }
            budget_lib::types::DeleteLineItemError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}

async fn add_line_item(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::AddLineItemRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.hashed_email.as_str()).await {
        return e;
    }
    if let Err(e) = verify_entitlement(req.hashed_email.as_str()).await {
        return e;
    }
    let res = budget_lib::add_line_item(req).await;
    match res {
        Ok(()) => http::StatusCode::CREATED,
        Err(e) => match e {
            budget_lib::types::AddLineItemError::UserDoesntExists() => http::StatusCode::NOT_FOUND,
            budget_lib::types::AddLineItemError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}

async fn edit_line_item(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::EditLineItemRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.hashed_email.as_str()).await {
        return e;
    }
    if let Err(e) = verify_entitlement(req.hashed_email.as_str()).await {
        return e;
    }
    let res = budget_lib::edit_line_item(req).await;
    match res {
        Ok(()) => http::StatusCode::OK,
        Err(e) => match e {
            budget_lib::types::EditLineItemError::UserDoesntExists() => http::StatusCode::NOT_FOUND,
            budget_lib::types::EditLineItemError::LineItemDoesntExist() => {
                http::StatusCode::NOT_FOUND
            }
            budget_lib::types::EditLineItemError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}

async fn add_category(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::AddCategoryRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.hashed_email.as_str()).await {
        return e;
    }
    if let Err(e) = verify_entitlement(req.hashed_email.as_str()).await {
        return e;
    }
    let res = budget_lib::add_category(req).await;
    match res {
        Ok(()) => http::StatusCode::CREATED,
        Err(e) => match e {
            budget_lib::types::AddCategoryError::UserDoesntExists() => http::StatusCode::NOT_FOUND,
            budget_lib::types::AddCategoryError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}

async fn clone_month(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::CloneMonthRequest>,
) -> http::StatusCode {
    if let Err(e) = verify_auth(headers, &req.hashed_email.as_str()).await {
        return e;
    }
    if let Err(e) = verify_entitlement(req.hashed_email.as_str()).await {
        return e;
    }
    let res = budget_lib::clone_last_month(req).await;
    match res {
        Ok(()) => http::StatusCode::CREATED,
        Err(e) => match e {
            budget_lib::types::CloneMonthError::UserDoesntExists() => http::StatusCode::NOT_FOUND,
            budget_lib::types::CloneMonthError::SourceMonthDoesntExist() => {
                http::StatusCode::NOT_FOUND
            }
            budget_lib::types::CloneMonthError::Internal(_) => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        },
    }
}

async fn export_csv(
    headers: axum::http::HeaderMap,
    axum::extract::Query(req): axum::extract::Query<budget_lib::types::GetFullBudgetRequest>,
) -> (http::StatusCode, String) {
    if let Err(e) = verify_auth(headers, &req.hashed_email).await {
        return (e, "".to_string());
    }
    if let Err(e) = verify_entitlement(&req.hashed_email).await {
        return (e, "".to_string());
    }
    let res = budget_lib::get_full_budget(req).await;
    match res {
        Ok(csv) => (http::StatusCode::OK, csv.as_csv()),
        Err(e) => (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn get_subscription(
    headers: axum::http::HeaderMap,
    axum::extract::Query(req): axum::extract::Query<budget_lib::types::GetSubscriptionRequest>,
) -> (http::StatusCode, axum::Json<serde_json::Value>) {
    if let Err(e) = verify_auth(headers, req.hashed_email.as_str()).await {
        return (
            e,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        );
    }
    match budget_lib::get_subscription(req).await {
        Ok(sub) => {
            let body = serde_json::json!({
                "status": sub.status,
                "entitlement": sub.entitlement(),
                "trial_ends_at": sub.trial_ends_at,
                "current_period_end": sub.current_period_end,
                "cancel_at_period_end": sub.cancel_at_period_end,
            });
            (http::StatusCode::OK, axum::Json(body))
        }
        Err(budget_lib::types::GetSubscriptionError::UserDoesntExists())
        | Err(budget_lib::types::GetSubscriptionError::SubscriptionDoesntExists()) => (
            http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        ),
        Err(budget_lib::types::GetSubscriptionError::Internal(_)) => (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        ),
    }
}

/// Starts a Stripe Checkout Session for the monthly plan and returns its hosted
/// URL for the browser to redirect to. This is the button a user lands on once
/// their free trial lapses (a `402` from the entitlement check).
async fn create_checkout_session(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::StartCheckoutRequest>,
) -> (http::StatusCode, axum::Json<serde_json::Value>) {
    if let Err(e) = verify_auth(headers, req.hashed_email.as_str()).await {
        return (
            e,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        );
    }
    match budget_lib::start_subscription_checkout(req).await {
        Ok(res) => (
            http::StatusCode::OK,
            axum::Json(serde_json::to_value(res).unwrap_or_default()),
        ),
        Err(budget_lib::types::StartCheckoutError::UserDoesntExists()) => (
            http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        ),
        Err(budget_lib::types::StartCheckoutError::StripeNotConfigured()) => {
            eprintln!(
                "Checkout requested but Stripe is not configured: set STRIPE_SECRET_KEY and \
                 STRIPE_PRICE_ID in this service's environment"
            );
            (
                http::StatusCode::SERVICE_UNAVAILABLE,
                axum::Json(serde_json::from_str("{}").unwrap_or_default()),
            )
        }
        Err(budget_lib::types::StartCheckoutError::Stripe(msg)) => {
            eprintln!("Stripe checkout error: {msg}");
            (
                http::StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::from_str("{}").unwrap_or_default()),
            )
        }
        Err(budget_lib::types::StartCheckoutError::Internal(msg)) => {
            eprintln!("Checkout internal error: {msg}");
            (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::from_str("{}").unwrap_or_default()),
            )
        }
    }
}

/// Schedules the caller's paid subscription to stop renewing. They keep full
/// access until the end of the billing period they have already paid for.
async fn cancel_subscription(
    headers: axum::http::HeaderMap,
    axum::extract::Json(req): axum::extract::Json<budget_lib::types::CancelSubscriptionRequest>,
) -> (http::StatusCode, axum::Json<serde_json::Value>) {
    if let Err(e) = verify_auth(headers, req.hashed_email.as_str()).await {
        return (
            e,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        );
    }
    match budget_lib::cancel_subscription(req).await {
        Ok(res) => (
            http::StatusCode::OK,
            axum::Json(serde_json::to_value(res).unwrap_or_default()),
        ),
        Err(budget_lib::types::CancelSubscriptionError::UserDoesntExists()) => (
            http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        ),
        Err(budget_lib::types::CancelSubscriptionError::NotCancelable()) => (
            http::StatusCode::CONFLICT,
            axum::Json(serde_json::from_str("{}").unwrap_or_default()),
        ),
        Err(budget_lib::types::CancelSubscriptionError::StripeNotConfigured()) => {
            eprintln!(
                "Cancel requested but Stripe is not configured: set STRIPE_SECRET_KEY in this \
                 service's environment"
            );
            (
                http::StatusCode::SERVICE_UNAVAILABLE,
                axum::Json(serde_json::from_str("{}").unwrap_or_default()),
            )
        }
        Err(budget_lib::types::CancelSubscriptionError::Stripe(msg)) => {
            eprintln!("Stripe cancel error: {msg}");
            (
                http::StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::from_str("{}").unwrap_or_default()),
            )
        }
        Err(budget_lib::types::CancelSubscriptionError::Internal(msg)) => {
            eprintln!("Cancel internal error: {msg}");
            (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::from_str("{}").unwrap_or_default()),
            )
        }
    }
}

/// Receives Stripe webhooks. The body must be read raw so its bytes match what
/// Stripe signed, so this handler takes `Bytes` rather than `Json`. A bad or
/// missing signature is a `400`; anything we can't finish processing is a `500`
/// so Stripe retries; everything else (including events we don't act on) is a
/// `200` acknowledgement.
async fn handle_stripe_webhook(
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> http::StatusCode {
    let secret = match std::env::var("STRIPE_WEBHOOK_SECRET") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("STRIPE_WEBHOOK_SECRET is not set; rejecting webhook");
            return http::StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    let signature = match headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s,
        None => return http::StatusCode::BAD_REQUEST,
    };

    let event = match budget_lib::stripe::construct_event(&body, signature, &secret) {
        Ok(event) => event,
        Err(e) => {
            eprintln!("Rejected Stripe webhook: {e}");
            return http::StatusCode::BAD_REQUEST;
        }
    };

    match event.event_type.as_str() {
        "checkout.session.completed" => {
            let session: budget_lib::stripe::CompletedCheckoutSession =
                match serde_json::from_value(event.data.object) {
                    Ok(session) => session,
                    Err(e) => {
                        eprintln!("checkout.session.completed with unexpected shape: {e}");
                        return http::StatusCode::BAD_REQUEST;
                    }
                };

            match budget_lib::record_completed_checkout(session).await {
                Ok(()) => http::StatusCode::OK,
                // Nothing actionable — acknowledge so Stripe stops retrying.
                Err(budget_lib::types::RecordCheckoutError::NoAccountReference())
                | Err(budget_lib::types::RecordCheckoutError::UserDoesntExists()) => {
                    eprintln!(
                        "checkout.session.completed {} could not be matched to an account",
                        event.id
                    );
                    http::StatusCode::OK
                }
                Err(budget_lib::types::RecordCheckoutError::Internal(msg)) => {
                    eprintln!("Failed to record checkout {}: {msg}", event.id);
                    http::StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
        event_type @ ("customer.subscription.updated" | "customer.subscription.deleted") => {
            let subscription: budget_lib::stripe::SubscriptionLifecycle =
                match serde_json::from_value(event.data.object) {
                    Ok(subscription) => subscription,
                    Err(e) => {
                        eprintln!("{event_type} with unexpected shape: {e}");
                        return http::StatusCode::BAD_REQUEST;
                    }
                };
            let ended = event_type == "customer.subscription.deleted";

            match budget_lib::apply_subscription_lifecycle(subscription, ended).await {
                Ok(()) => http::StatusCode::OK,
                // No account to route to (e.g. a subscription created before we
                // started stamping metadata) — acknowledge and move on.
                Err(budget_lib::types::SyncSubscriptionError::NoAccountReference())
                | Err(budget_lib::types::SyncSubscriptionError::UserDoesntExists()) => {
                    eprintln!(
                        "{event_type} {} could not be matched to an account",
                        event.id
                    );
                    http::StatusCode::OK
                }
                Err(budget_lib::types::SyncSubscriptionError::Internal(msg)) => {
                    eprintln!("Failed to apply {event_type} {}: {msg}", event.id);
                    http::StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
        // Acknowledged but not acted on.
        _ => http::StatusCode::OK,
    }
}

/// Rejects the request unless the user currently has budget access: a lifetime
/// license, an active paid subscription, or a free trial that has not run out.
/// Auth is assumed to have already been checked by the caller.
async fn verify_entitlement(hashed_email: &str) -> Result<(), http::StatusCode> {
    let req = budget_lib::types::GetSubscriptionRequest {
        hashed_email: hashed_email.to_string(),
    };
    match budget_lib::get_subscription(req).await {
        Ok(sub) if sub.has_access() => Ok(()),
        Ok(_) => Err(http::StatusCode::PAYMENT_REQUIRED),
        Err(budget_lib::types::GetSubscriptionError::UserDoesntExists())
        | Err(budget_lib::types::GetSubscriptionError::SubscriptionDoesntExists()) => {
            Err(http::StatusCode::NOT_FOUND)
        }
        Err(budget_lib::types::GetSubscriptionError::Internal(_)) => {
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn verify_auth(
    headers: axum::http::HeaderMap,
    hashed_email: &str,
) -> Result<(), http::StatusCode> {
    let auth = headers.get("Authorization");
    match auth {
        None => {
            return Err(http::StatusCode::UNAUTHORIZED);
        }
        Some(token) => {
            let token_str = token.to_str().unwrap_or("");
            let res = budget_lib::check_token(hashed_email, token_str).await;
            match res {
                Err(_) => {
                    return Err(http::StatusCode::UNAUTHORIZED);
                }
                Ok(()) => Ok(()),
            }
        }
    }
}
