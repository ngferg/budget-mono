use crate::dao::Dao;
use crate::types;

pub(crate) struct SqliteDao {}

impl Dao for SqliteDao {
    fn create_user(
        &self,
        req: &types::dao::CreateUserRequest,
    ) -> Result<(), types::dao::CreateUserError> {
        println!("Got a request to create a user: {}", req.email);
        if !req.email.contains("@") {
            return Err(types::dao::CreateUserError::EmailImproperlyFormatted());
        }
        Ok(())
    }
}

impl SqliteDao {
    pub(crate) fn new() -> Self {
        SqliteDao {}
    }
}
