use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

mod dao;
pub mod types;

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
