use std::sync::{Arc, Mutex};

use crate::dao::Dao;
use crate::types;

pub(crate) struct SqliteDao<CON: SqLiteConn> {
    conn: Arc<Mutex<CON>>,
}

impl<CON: SqLiteConn> Dao for SqliteDao<CON> {
    fn create_user(&self, req: &types::CreateUserRequest) -> Result<(), types::CreateUserError> {
        self.conn
            .lock()
            .map_err(|_| types::CreateUserError::Internal("Failed to lock conn".to_string()))?
            .create_db(req.hashed_email.clone())
            .map_err(|e| match e {
                DaoError::DbAlreadyExists() => types::CreateUserError::UserAlreadyExists(),
                _ => types::CreateUserError::Internal("Failed to create DB".to_string()),
            })
    }

    fn add_line_item(
        &self,
        req: &types::AddLineItemRequest,
    ) -> Result<(), types::AddLineItemError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| types::AddLineItemError::Internal("Failed to lock conn".to_string()))?
            .get_db(req.hashed_email.clone())
            .map_err(|_| types::AddLineItemError::UserDoesntExists())?;
        let mut insert_stmt = conn
            .prepare("INSERT INTO line_items (description, amount, category, budget_year, budget_month) VALUES (?, ?, ?, ?, ?)")
            .map_err(|_| {
                types::AddLineItemError::Internal("Failed to prepare insert statement".to_string())
            })?;
        insert_stmt
            .execute(rusqlite::params![
                req.description,
                req.amount,
                req.category_id,
                req.year,
                req.month
            ])
            .map_err(|e| {
                types::AddLineItemError::Internal(format!("Failed to insert line_item: {e}"))
            })?;
        Ok(())
    }

    fn edit_line_item(
        &self,
        req: &types::EditLineItemRequest,
    ) -> Result<(), types::EditLineItemError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| types::EditLineItemError::Internal("Failed to lock conn".to_string()))?
            .get_db(req.hashed_email.clone())
            .map_err(|_| types::EditLineItemError::UserDoesntExists())?;
        let mut update_stmt = conn
            .prepare("UPDATE line_items SET description = ?, amount = ? WHERE id = ?")
            .map_err(|_| {
                types::EditLineItemError::Internal("Failed to prepare update statement".to_string())
            })?;
        update_stmt
            .execute(rusqlite::params![req.description, req.amount, req.item_id])
            .map_err(|e| {
                types::EditLineItemError::Internal(format!("Failed to update line_item: {e}"))
            })?;
        Ok(())
    }

    fn delete_user(&self, req: &types::DeleteUserRequest) -> Result<(), types::DeleteUserError> {
        self.conn
            .lock()
            .map_err(|_| types::DeleteUserError::Internal("Failed to lock conn".to_string()))?
            .delete_db(req.hashed_email.clone())
            .map_err(|_| types::DeleteUserError::Internal("Failed to delete user".to_string()))
    }

    fn delete_line_item(
        &self,
        req: &types::DeleteLineItemRequest,
    ) -> Result<(), types::DeleteLineItemError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| types::DeleteLineItemError::Internal("Failed to lock conn".to_string()))?
            .get_db(req.hashed_email.clone())
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
        let conn = self
            .conn
            .lock()
            .map_err(|_| types::GetBudgetError::Internal("Failed to lock conn".to_string()))?
            .get_db(req.hashed_email.clone())
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
            .query_map(rusqlite::params![req.year, req.month.inner()], |row| {
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

        let last_month = if req.month.inner() == 1 {
            12
        } else {
            req.month.inner() - 1
        };
        let last_year = if req.month.inner() == 1 {
            req.year - 1
        } else {
            req.year
        };
        // Check if there is at least one line item in last month's table
        let mut last_month_stmt = conn
            .prepare("SELECT COUNT(1) FROM line_items WHERE budget_year = ? AND budget_month = ?")
            .map_err(|_| {
                types::GetBudgetError::Internal("Failed to query last month line_items".to_string())
            })?;
        let last_month_count: u64 = last_month_stmt
            .query_row(rusqlite::params![last_year, last_month], |row| row.get(0))
            .unwrap_or(0);
        let last_month_clonable = last_month_count > 0 && budget.iter().all(|cat| cat.1.len() == 0);

        Ok(types::GetBudgetResponse {
            categories,
            budget,
            last_month_clonable,
        })
    }

    fn add_category(&self, req: &types::AddCategoryRequest) -> Result<(), types::AddCategoryError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| types::AddCategoryError::Internal("Failed to lock conn".to_string()))?
            .get_db(req.hashed_email.clone())
            .map_err(|_| types::AddCategoryError::UserDoesntExists())?;

        let mut insert_stmt = conn
            .prepare("INSERT INTO categories (category, is_expense) VALUES (?, ?)")
            .map_err(|_| {
                types::AddCategoryError::Internal("Failed to prepare insert statement".to_string())
            })?;
        insert_stmt
            .execute(rusqlite::params![req.category, req.is_expense])
            .map_err(|e| {
                types::AddCategoryError::Internal(format!("Failed to insert category: {e}"))
            })?;
        Ok(())
    }

    fn clone_month(&self, req: &types::CloneMonthRequest) -> Result<(), types::CloneMonthError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| types::CloneMonthError::Internal("failed to lock conn".to_string()))?
            .get_db(req.hashed_email.clone())
            .map_err(|_| types::CloneMonthError::UserDoesntExists())?;

        let mut select_stmt = conn
            .prepare("SELECT description, amount, category FROM line_items WHERE budget_year = ? AND budget_month = ?")
            .map_err(|_| {
                types::CloneMonthError::Internal("Failed to prepare select statement".to_string())
            })?;
        let line_item_iter = select_stmt
            .query_map(
                rusqlite::params![req.source_year, req.source_month.inner()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, u64>(1)?,
                        row.get::<_, u64>(2)?,
                    ))
                },
            )
            .map_err(|e| {
                types::CloneMonthError::Internal(format!("Failed to query line_items: {e}"))
            })?;

        let mut insert_stmt = conn
            .prepare("INSERT INTO line_items (description, amount, category, budget_year, budget_month) VALUES (?, ?, ?, ?, ?)")
            .map_err(|_| {
                types::CloneMonthError::Internal("Failed to prepare insert statement".to_string())
            })?;

        for line_item_res in line_item_iter {
            let (description, amount, category) = line_item_res.map_err(|e| {
                types::CloneMonthError::Internal(format!("Failed to read line_item: {e}"))
            })?;
            insert_stmt
                .execute(rusqlite::params![
                    description,
                    amount,
                    category,
                    req.target_year,
                    req.target_month.inner()
                ])
                .map_err(|e| {
                    types::CloneMonthError::Internal(format!("Failed to insert line_item: {e}"))
                })?;
        }

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum DaoError {
    #[error("Failed to insantiate DAO: {0}")]
    FailedToCreate(String),
    #[error("Failed to Delete DB")]
    FailedToDelete(),
    #[error("DB doesn't exists")]
    DbDoesntExist(),
    #[error("DB already exists")]
    DbAlreadyExists(),
}

impl<CON: SqLiteConn> SqliteDao<CON> {
    pub(crate) fn new(conn: Arc<Mutex<CON>>) -> Self {
        SqliteDao { conn }
    }
}

pub(crate) trait SqLiteConn {
    fn get_db(&self, hashed_email: String) -> Result<rusqlite::Connection, DaoError>;
    fn create_db(&self, hashed_email: String) -> Result<(), DaoError>;
    fn delete_db(&self, hashed_email: String) -> Result<(), DaoError>;
}

pub(crate) struct RealSqliteConn {
    db_folder: String,
}

impl RealSqliteConn {
    pub(crate) fn try_new() -> Result<Self, DaoError> {
        let db_folder = std::env::var("SQLITE_DB_PATH")
            .map_err(|_| DaoError::FailedToCreate("SQLITE_DB_PATH env var not set".to_string()))?;
        Ok(RealSqliteConn { db_folder })
    }
}

impl SqLiteConn for RealSqliteConn {
    fn get_db(&self, hashed_email: String) -> Result<rusqlite::Connection, DaoError> {
        let sqlite_file_path = format!("{}/{}.db", self.db_folder, hashed_email);
        if !std::path::Path::new(&sqlite_file_path).exists() {
            return Err(DaoError::DbDoesntExist());
        }
        let conn =
            rusqlite::Connection::open(sqlite_file_path).map_err(|_| DaoError::DbDoesntExist())?;
        Ok(conn)
    }

    fn create_db(&self, hashed_email: String) -> Result<(), DaoError> {
        let sqlite_file_path = format!("{}/{}.db", self.db_folder, hashed_email);
        let sqlite_file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(sqlite_file_path.clone());
        match sqlite_file {
            Ok(_) => {
                let conn = rusqlite::Connection::open(sqlite_file_path).map_err(|_| {
                    DaoError::FailedToCreate("Failed to create user database".to_string())
                })?;

                let ddl = std::fs::read_to_string(format!("{}/USER_DDL.sql", self.db_folder))
                    .map_err(|_| DaoError::FailedToCreate("DDL sql is missing".to_string()))?;
                conn.execute_batch(ddl.as_str())
                    .map_err(|_| DaoError::FailedToCreate("Failed to execute DDL".to_string()))?;
                Ok(())
            }
            Err(_) => Err(DaoError::DbAlreadyExists()),
        }
    }

    fn delete_db(&self, hashed_email: String) -> Result<(), DaoError> {
        let sqlite_file_path = format!("{}/{}.db", self.db_folder, hashed_email);
        let _ = std::fs::remove_file(sqlite_file_path).map_err(|_| DaoError::FailedToDelete())?;
        Ok(())
    }
}
