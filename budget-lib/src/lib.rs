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
