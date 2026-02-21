mod dao;
pub mod types;

pub async fn create_user(
    create_user_request: types::CreateUserRequest,
) -> Result<(), types::CreateUserError> {
    use dao::Dao as dao_trait;

    let dao = dao::sqlite_dao::SqliteDao::try_new()
        .inspect_err(|e| eprintln!("Failed to create sqlite dao: {}", e.to_string()))
        .unwrap();
    let _ = dao.create_user(&create_user_request)?;
    Ok(())
}

pub async fn delete_user(
    del_user_request: types::DeleteUserRequest,
) -> Result<(), types::DeleteUserError> {
    use dao::Dao as dao_trait;

    let dao = dao::sqlite_dao::SqliteDao::try_new()
        .inspect_err(|e| eprintln!("Failed to create sqlite dao: {}", e.to_string()))
        .unwrap();
    let _ = dao.delete_user(&del_user_request)?;
    Ok(())
}

pub async fn delete_line_item(
    req: types::DeleteLineItemRequest,
) -> Result<(), types::DeleteLineItemError> {
    use dao::Dao as dao_trait;

    let dao = dao::sqlite_dao::SqliteDao::try_new()
        .inspect_err(|e| eprintln!("Failed to create sqlite dao: {}", e.to_string()))
        .unwrap();
    let _ = dao.delete_line_item(&req)?;
    Ok(())
}

pub async fn add_line_item(req: types::AddLineItemRequest) -> Result<(), types::AddLineItemError> {
    use dao::Dao as dao_trait;

    let dao = dao::sqlite_dao::SqliteDao::try_new()
        .inspect_err(|e| eprintln!("Failed to create sqlite dao: {}", e.to_string()))
        .unwrap();
    let _ = dao.add_line_item(&req)?;
    Ok(())
}

pub async fn get_budget(
    get_budget_request: types::GetBudgetRequest,
) -> Result<types::GetBudgetResponse, types::GetBudgetError> {
    use dao::Dao as dao_trait;

    get_budget_request.validate()?;

    let dao = dao::sqlite_dao::SqliteDao::try_new()
        .inspect_err(|e| eprintln!("Failed to create sqlite dao: {}", e.to_string()))
        .unwrap();
    let res = dao.get_budget(&get_budget_request)?;
    Ok(res)
}

pub async fn edit_line_item(
    req: types::EditLineItemRequest,
) -> Result<(), types::EditLineItemError> {
    use dao::Dao as dao_trait;

    let dao = dao::sqlite_dao::SqliteDao::try_new()
        .inspect_err(|e| eprintln!("Failed to create sqlite dao: {}", e.to_string()))
        .unwrap();
    let _ = dao.edit_line_item(&req)?;
    Ok(())
}

pub async fn clone_last_month(req: types::CloneMonthRequest) -> Result<(), types::CloneMonthError> {
    use dao::Dao as dao_trait;

    let dao = dao::sqlite_dao::SqliteDao::try_new()
        .inspect_err(|e| eprintln!("Failed to create sqlite dao: {}", e.to_string()))
        .unwrap();
    let _ = dao.clone_month(&req)?;
    Ok(())
}

pub async fn check_token(email: &str, token: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    client
        .post("http://localhost:3001/verify_token")
        .json(&types::VerifyTokenRequest {
            email: email.to_string(),
            token: token.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Failed to send request to auth service: {}", e))?
        .error_for_status()
        .map_err(|e| format!("Token verification failed: {}", e))?;
    Ok(())
}
