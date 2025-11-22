use crate::dao::Dao;
use crate::types;
use chrono::Datelike;

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
        let sqlite_file_path = format!("dbs/{}.db", email_sha);
        let sqlite_file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(sqlite_file_path.clone());
        match sqlite_file {
            Ok(_) => {
                let conn = rusqlite::Connection::open(sqlite_file_path)
                    .expect("Failed to open checked sqlite db");

                let ddl = std::fs::read_to_string("dbs/USER_DDL.sql").expect("DDL sql is missing");
                conn.execute_batch(ddl.as_str()).unwrap();

                let current_date = chrono::Local::now();
                let year = current_date.year();
                let month = current_date.month();
                conn.execute(
                    format!(
                        "INSERT INTO budget (year, month) VALUES ({}, {})",
                        year, month
                    )
                    .as_str(),
                    (),
                )
                .unwrap();
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
