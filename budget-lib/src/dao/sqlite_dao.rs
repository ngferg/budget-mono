use crate::dao::Dao;
use crate::types;
use chrono::Datelike;

pub(crate) struct SqliteDao {}

impl Dao for SqliteDao {
    fn create_user(&self, req: &types::CreateUserRequest) -> Result<(), types::CreateUserError> {
        println!("Got a request to create a user: {}", req.email);
        let db_folder = std::env::var("SQLITE_DB_PATH").expect("SQLITE_DB_PATH env var not set");
        if !req.email.contains("@") {
            return Err(types::CreateUserError::EmailImproperlyFormatted());
        }
        let email_sha = sha256::digest(req.email.clone());
        println!("Attempting to create db: {}", email_sha);
        let sqlite_file_path = format!("{}/{}.db", db_folder, email_sha);
        let sqlite_file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(sqlite_file_path.clone());
        match sqlite_file {
            Ok(_) => {
                let conn = rusqlite::Connection::open(sqlite_file_path)
                    .expect("Failed to open checked sqlite db");

                let ddl = std::fs::read_to_string(format!("{}/USER_DDL.sql", db_folder))
                    .expect("DDL sql is missing");
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
                Err(types::CreateUserError::UserAlreadyExists())
            }
        }
    }
}

impl SqliteDao {
    pub(crate) fn new() -> Self {
        SqliteDao {}
    }
}
