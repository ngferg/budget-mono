use crate::dao::Dao;
use crate::types;

pub(crate) struct SqliteDao {
    db_folder: String,
}

impl Dao for SqliteDao {
    fn create_user(&self, req: &types::CreateUserRequest) -> Result<(), types::CreateUserError> {
        println!("Got a request to create a user: {}", req.email);
        if !req.email.contains("@") {
            return Err(types::CreateUserError::EmailImproperlyFormatted());
        }
        let email_sha = sha256::digest(req.email.clone());
        println!("Attempting to create db: {}", email_sha);
        let sqlite_file_path = format!("{}/{}.db", self.db_folder, email_sha);
        let sqlite_file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(sqlite_file_path.clone());
        match sqlite_file {
            Ok(_) => {
                let conn = rusqlite::Connection::open(sqlite_file_path)
                    .expect("Failed to open checked sqlite db");

                let ddl = std::fs::read_to_string(format!("{}/USER_DDL.sql", self.db_folder))
                    .expect("DDL sql is missing");
                conn.execute_batch(ddl.as_str()).unwrap();
                Ok(())
            }
            Err(e) => {
                println!("Fail: {}", e);
                Err(types::CreateUserError::UserAlreadyExists())
            }
        }
    }

    fn delete_user(&self, req: &types::DeleteUserRequest) -> Result<(), types::DeleteUserError> {
        println!("Got a request to delte user: {}", req.email);
        let email_sha = sha256::digest(req.email.clone());
        println!("Attempting to delete db: {}", email_sha);
        let sqlite_file_path = format!("{}/{}.db", self.db_folder, email_sha);
        let res = std::fs::remove_file(sqlite_file_path);
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(types::DeleteUserError::Internal(e.to_string())),
        }
    }

    fn delete_line_item(
        &self,
        req: &types::DeleteLineItemRequest,
    ) -> Result<(), types::DeleteLineItemError> {
        println!("Got a request to delete line item: {:?}", req);
        let email_sha = sha256::digest(req.email.clone());
        let sqlite_file_path = format!("{}/{}.db", self.db_folder, email_sha);
        if !std::path::Path::new(&sqlite_file_path).exists() {
            return Err(types::DeleteLineItemError::UserDoesntExists());
        }
        let conn = rusqlite::Connection::open(sqlite_file_path)
            .map_err(|_| types::DeleteLineItemError::UserDoesntExists())?;
        let mut delete_stmt = conn
            .prepare("DELETE FROM line_items WHERE budget_year = ? AND budget_month = ? and id = ?")
            .map_err(|_| {
                types::DeleteLineItemError::Internal("Failed to delete line_items".to_string())
            })?;
        delete_stmt
            .execute(rusqlite::params![req.year, req.month, req.item_id])
            .map_err(|_| {
                types::DeleteLineItemError::Internal(format!(
                    "Failed to delete line_item for year {}, month {}, id {}",
                    req.year, req.month, req.item_id
                ))
            })?;
        Ok(())
    }

    fn get_budget(
        &self,
        req: &types::GetBudgetRequest,
    ) -> Result<types::GetBudgetResponse, types::GetBudgetError> {
        println!("Got a request to fetch budget: {:?}", req);
        let email_sha = sha256::digest(req.email.clone());
        let sqlite_file_path = format!("{}/{}.db", self.db_folder, email_sha);
        if !std::path::Path::new(&sqlite_file_path).exists() {
            return Err(types::GetBudgetError::UserDoesntExists());
        }
        let conn = rusqlite::Connection::open(sqlite_file_path)
            .map_err(|_| types::GetBudgetError::UserDoesntExists())?;
        let mut category_stmt = conn.prepare("SELECT * FROM categories").map_err(|_| {
            types::GetBudgetError::Internal("Failed to select categories".to_string())
        })?;
        let category_iter = category_stmt
            .query_map(rusqlite::params![], |row| {
                Ok(types::Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category_type: match row.get(2)? {
                        true => types::CategoryType::Expense,
                        false => types::CategoryType::Income,
                    },
                })
            })
            .map_err(|_| {
                types::GetBudgetError::Internal("Failed to query category tables".to_string())
            })?;
        let categories = category_iter
            .collect::<Result<Vec<types::Category>, _>>()
            .map_err(|e| {
                types::GetBudgetError::Internal(format!("Failed to query category table {}", e))
            })?;

        let mut budget = std::collections::HashMap::new();
        categories.iter().for_each(|cat| {
            budget.insert(cat.id, vec![]);
        });

        let mut budget_stmt = conn
            .prepare("SELECT * FROM line_items WHERE budget_year = ? AND budget_month = ?")
            .map_err(|_| {
                types::GetBudgetError::Internal("Failed to select line_items".to_string())
            })?;
        let budget_iter = budget_stmt
            .query_map(rusqlite::params![req.year, req.month], |row| {
                Ok(types::LineItem {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    amount: row.get(2)?,
                    category: row.get(3)?,
                })
            })
            .map_err(|e| {
                types::GetBudgetError::Internal(format!("Failed to query line_item table {}", e))
            })?;
        let line_items = budget_iter
            .collect::<Result<Vec<types::LineItem>, _>>()
            .map_err(|e| {
                types::GetBudgetError::Internal(format!("Failed to query line_item table {}", e))
            })?;
        line_items.into_iter().for_each(|li| {
            if let Some(lis) = budget.get_mut(&li.category) {
                lis.push(li);
            }
        });

        Ok(types::GetBudgetResponse { categories, budget })
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum DaoError {
    #[error("Failed to insantiate DAO: {0}")]
    FailedToCreate(String),
}

impl SqliteDao {
    pub(crate) fn try_new() -> Result<Self, DaoError> {
        let db_folder = std::env::var("SQLITE_DB_PATH")
            .map_err(|_| DaoError::FailedToCreate("SQLITE_DB_PATH env var not set".to_string()))?;
        Ok(SqliteDao { db_folder })
    }
}
