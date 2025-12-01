mod dao;
pub mod types;

pub async fn create_user(
    create_user_request: types::CreateUserRequest,
) -> Result<(), types::CreateUserError> {
    use dao::Dao as dao_trait;

    let dao = dao::sqlite_dao::SqliteDao::new();
    let _ = dao.create_user(&create_user_request)?;
    Ok(())
}
