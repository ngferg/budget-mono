use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

mod dao;
pub mod stripe;
pub mod types;

/// Where Stripe Checkout returns the customer. Overridable so dev, preview and
/// prod each send the browser back to the right origin.
fn public_base_url() -> String {
    std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "http://localhost:5173".to_string())
}

pub async fn get_full_budget(
    req: types::GetFullBudgetRequest,
) -> Result<types::FullBudgetResponse, types::GetBudgetError> {
    use dao::Dao as dao_trait;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::GetBudgetError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    let line_items = dao
        .get_all_line_items(&types::GetAllLineItemsRequest {
            hashed_email: req.hashed_email.clone(),
        })
        .map_err(|e| types::GetBudgetError::Internal(format!("Failed to get line items: {e}")))?;
    let categories = dao
        .get_all_categories(&types::GetAllCategoriesRequest {
            hashed_email: req.hashed_email,
        })
        .map_err(|e| types::GetBudgetError::Internal(format!("Failed to get categories: {e}")))?;

    let mut budget = BTreeMap::new();
    for line_item in line_items {
        let category = categories
            .get(&line_item.category)
            .cloned()
            .unwrap_or(types::Category {
                id: u64::MAX,
                name: "unknown".to_string(),
                category_type: types::CategoryType::Expense,
            });
        let category_map = budget
            .entry((line_item.year, line_item.month))
            .or_insert_with(BTreeMap::new);
        category_map
            .entry(category)
            .or_insert_with(Vec::new)
            .push(types::LineItem {
                id: line_item.id,
                description: line_item.description,
                amount: line_item.amount,
                category: line_item.category,
            });
    }

    Ok(types::FullBudgetResponse { budget })
}

pub async fn create_user(
    create_user_request: types::CreateUserRequest,
) -> Result<(), types::CreateUserError> {
    use dao::Dao as dao_trait;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::CreateUserError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    let _ = dao.create_user(&create_user_request)?;
    Ok(())
}

pub async fn get_subscription(
    req: types::GetSubscriptionRequest,
) -> Result<types::Subscription, types::GetSubscriptionError> {
    use dao::Dao as dao_trait;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::GetSubscriptionError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    dao.get_subscription(&req)
}

/// Opens a Stripe Checkout Session so the user can start paying for the monthly
/// plan, and hands back the hosted URL to redirect them to. The account's hashed
/// email rides along as the session's `client_reference_id` so the webhook can
/// match the payment back to this database.
pub async fn start_subscription_checkout(
    req: types::StartCheckoutRequest,
) -> Result<types::CheckoutSessionResponse, types::StartCheckoutError> {
    use dao::Dao as dao_trait;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::StartCheckoutError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));

    // Confirms the account exists (and surfaces a clean 404) before we spend a
    // round trip on Stripe.
    dao.get_subscription(&types::GetSubscriptionRequest {
        hashed_email: req.hashed_email.clone(),
    })
    .map_err(|e| match e {
        types::GetSubscriptionError::UserDoesntExists()
        | types::GetSubscriptionError::SubscriptionDoesntExists() => {
            types::StartCheckoutError::UserDoesntExists()
        }
        types::GetSubscriptionError::Internal(msg) => types::StartCheckoutError::Internal(msg),
    })?;

    let price_id = std::env::var("STRIPE_PRICE_ID")
        .map_err(|_| types::StartCheckoutError::StripeNotConfigured())?;
    let base = public_base_url();
    let success_url = format!("{base}/?checkout=success&session_id={{CHECKOUT_SESSION_ID}}");
    let cancel_url = format!("{base}/?checkout=cancelled");

    let client = stripe::StripeClient::from_env().map_err(|e| match e {
        stripe::StripeError::MissingApiKey => types::StartCheckoutError::StripeNotConfigured(),
        other => types::StartCheckoutError::Stripe(other.to_string()),
    })?;
    let session = client
        .create_checkout_session(&price_id, &req.hashed_email, &success_url, &cancel_url)
        .await
        .map_err(|e| types::StartCheckoutError::Stripe(e.to_string()))?;

    Ok(types::CheckoutSessionResponse { url: session.url })
}

/// Applies a verified `checkout.session.completed` event: flips the paying
/// account to `active` and stores Stripe's customer/subscription identifiers.
/// Safe to call more than once for the same session.
pub async fn record_completed_checkout(
    session: stripe::CompletedCheckoutSession,
) -> Result<(), types::RecordCheckoutError> {
    use dao::Dao as dao_trait;

    let hashed_email = session
        .client_reference_id
        .clone()
        .ok_or(types::RecordCheckoutError::NoAccountReference())?;

    // Mirror the paid-through date so entitlement checks never have to call
    // Stripe. Purely advisory — a failure here just leaves the column as-is.
    let current_period_end = match (&session.subscription, stripe::StripeClient::from_env()) {
        (Some(subscription_id), Ok(client)) => {
            client.subscription_period_end(subscription_id).await
        }
        _ => None,
    };

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::RecordCheckoutError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    dao.activate_subscription(&types::ActivateSubscriptionRequest {
        hashed_email,
        stripe_customer_id: session.customer,
        stripe_subscription_id: session.subscription,
        stripe_payment_intent_id: session.payment_intent,
        current_period_end,
    })
}

pub async fn delete_user(
    del_user_request: types::DeleteUserRequest,
) -> Result<(), types::DeleteUserError> {
    use dao::Dao as dao_trait;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::DeleteUserError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    let _ = dao.delete_user(&del_user_request)?;
    Ok(())
}

pub async fn delete_line_item(
    req: types::DeleteLineItemRequest,
) -> Result<(), types::DeleteLineItemError> {
    use dao::Dao as dao_trait;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::DeleteLineItemError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    let _ = dao.delete_line_item(&req)?;
    Ok(())
}

pub async fn add_line_item(req: types::AddLineItemRequest) -> Result<(), types::AddLineItemError> {
    use dao::Dao as dao_trait;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::AddLineItemError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    let _ = dao.add_line_item(&req)?;
    Ok(())
}

pub async fn get_budget(
    get_budget_request: types::GetBudgetRequest,
) -> Result<types::GetBudgetResponse, types::GetBudgetError> {
    use dao::Dao as dao_trait;

    get_budget_request.validate()?;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::GetBudgetError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    let res = dao.get_budget(&get_budget_request)?;
    Ok(res)
}

pub async fn edit_line_item(
    req: types::EditLineItemRequest,
) -> Result<(), types::EditLineItemError> {
    use dao::Dao as dao_trait;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::EditLineItemError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    let _ = dao.edit_line_item(&req)?;
    Ok(())
}

pub async fn add_category(req: types::AddCategoryRequest) -> Result<(), types::AddCategoryError> {
    use dao::Dao as dao_trait;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::AddCategoryError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    let _ = dao.add_category(&req)?;
    Ok(())
}

pub async fn clone_last_month(req: types::CloneMonthRequest) -> Result<(), types::CloneMonthError> {
    use dao::Dao as dao_trait;

    let conn = dao::sqlite_dao::RealSqliteConn::try_new().map_err(|e| {
        types::CloneMonthError::Internal(format!("Failed to create sqlite dao: {e}"))
    })?;
    let dao = dao::sqlite_dao::SqliteDao::new(Arc::new(Mutex::new(conn)));
    let _ = dao.clone_month(&req)?;
    Ok(())
}

pub async fn check_token(hashed_email: &str, token: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    client
        .post("http://localhost:3001/verify_token")
        .json(&types::VerifyTokenRequest {
            hashed_email: hashed_email.to_string(),
            token: token.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Failed to send request to auth service: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Token verification failed: {}", e))?;
    Ok(())
}
