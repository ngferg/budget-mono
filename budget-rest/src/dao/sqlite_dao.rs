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
        let email_sha = sha256::digest(req.email.clone());
        println!("Attempting to create db: {}", email_sha);
        std::fs::create_dir_all("dbs")
            .map_err(|e| types::dao::CreateUserError::Internal(e.to_string()))?;
        let file = std::fs::OpenOptions::new().write(true)
            .create_new(true)
            .open(format!("dbs/{}.db", email_sha));
        match file {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                println!("Fail: {}", e);
                Err(types::dao::CreateUserError::UserAlreadyExists())
            }
        }
    }
}

impl SqliteDao {
    pub(crate) fn new() -> Self {
        SqliteDao {}
    }
}
