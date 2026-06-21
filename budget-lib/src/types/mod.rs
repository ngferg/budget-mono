pub(crate) mod dao;
use std::collections::{self, BTreeMap};

use chrono::Datelike;

#[derive(thiserror::Error, Debug)]
pub enum CreateUserError {
    #[error("User already exists")]
    UserAlreadyExists(),
    #[error("Internal Error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteUserError {
    #[error("User doesn't exists")]
    UserDoesntExists(),
    #[error("Internal Error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum GetBudgetError {
    #[error("User doesn't exists")]
    UserDoesntExists(),
    #[error("Budget doesn't exists")]
    BudgetDoesntExists(),
    #[error("Date Error: {0}")]
    DateError(DateError),
    #[error("Internal Error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteLineItemError {
    #[error("User doesn't exists")]
    UserDoesntExists(),
    #[error("Budget doesn't exists")]
    LineItemDoesntExist(),
    #[error("Internal Error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum AddLineItemError {
    #[error("User doesn't exists")]
    UserDoesntExists(),
    #[error("Internal Error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum EditLineItemError {
    #[error("User doesn't exists")]
    UserDoesntExists(),
    #[error("Budget doesn't exists")]
    LineItemDoesntExist(),
    #[error("Internal Error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum AddCategoryError {
    #[error("User doesn't exists")]
    UserDoesntExists(),
    #[error("Internal Error: {0}")]
    Internal(String),
}

#[derive(thiserror::Error, Debug)]
pub enum CloneMonthError {
    #[error("User doesn't exists")]
    UserDoesntExists(),
    #[error("Source month doesn't exist")]
    SourceMonthDoesntExist(),
    #[error("Internal Error: {0}")]
    Internal(String),
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateUserRequest {
    pub hashed_email: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct DeleteUserRequest {
    pub hashed_email: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct DeleteLineItemRequest {
    pub item_id: u64,
    pub hashed_email: String,
    pub year: u32,
    pub month: u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct AddLineItemRequest {
    pub hashed_email: String,
    pub year: u32,
    pub month: u32,
    pub category_id: u64,
    pub description: String,
    pub amount: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct EditLineItemRequest {
    pub hashed_email: String,
    pub item_id: u64,
    pub description: String,
    pub amount: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct GetFullBudgetRequest {
    pub hashed_email: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct GetAllLineItemsRequest {
    pub hashed_email: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct GetAllCategoriesRequest {
    pub hashed_email: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct GetBudgetRequest {
    pub hashed_email: String,
    pub year: u32,
    pub month: Month,
}

impl GetBudgetRequest {
    pub fn validate(&self) -> Result<(), GetBudgetError> {
        let current_year = chrono::Utc::now().year() as u32;
        if self.year < 2026 || self.year > current_year + 3 {
            return Err(GetBudgetError::DateError(DateError::InvalidYear()));
        }
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct AddCategoryRequest {
    pub hashed_email: String,
    pub category: String,
    pub is_expense: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct CloneMonthRequest {
    pub hashed_email: String,
    pub source_year: u32,
    pub source_month: Month,
    pub target_year: u32,
    pub target_month: Month,
}

#[derive(Debug, serde::Serialize)]
pub struct FullBudgetResponse {
    pub budget: BTreeMap<(u32, u8), BTreeMap<Category, Vec<LineItem>>>,
}

impl FullBudgetResponse {
    pub fn as_csv(&self) -> String {
        let mut csv = String::new();
        csv.push_str("year,month,category,description,amount\n");
        for ((year, month), category_map) in &self.budget {
            for (category, line_items) in category_map {
                for line_item in line_items {
                    csv.push_str(&format!(
                        "{},{},{},{},{}\n",
                        year,
                        month,
                        category.name,
                        line_item.description,
                        (line_item.amount as f64 / 100.0) * category.multiplier()
                    ));
                }
            }
        }
        csv
    }
}

#[derive(Debug, serde::Serialize)]
pub struct GetBudgetResponse {
    pub categories: Vec<Category>,
    pub budget: collections::HashMap<u64, Vec<LineItem>>,
    pub last_month_clonable: bool,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct VerifyTokenRequest {
    pub hashed_email: String,
    pub token: String,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, serde::Serialize)]
pub enum CategoryType {
    Income,
    Expense,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, serde::Serialize)]
pub struct Category {
    pub id: u64,
    pub name: String,
    pub category_type: CategoryType,
}

impl Category {
    pub fn multiplier(&self) -> f64 {
        match self.category_type {
            CategoryType::Income => 1.0,
            CategoryType::Expense => -1.0,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct LineItem {
    pub id: u64,
    pub description: String,
    pub amount: u64,
    pub category: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct FullLineItem {
    pub month: u8,
    pub year: u32,
    pub id: u64,
    pub description: String,
    pub amount: u64,
    pub category: u64,
}

#[derive(thiserror::Error, Debug)]
pub enum DateError {
    #[error("Invalid year")]
    InvalidYear(),
    #[error("Invalid month")]
    InvalidMonth(),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "u8")]
pub struct Month(u8);
impl TryFrom<u8> for Month {
    type Error = DateError;

    fn try_from(raw: u8) -> Result<Self, Self::Error> {
        if raw < 1 || raw > 12 {
            Err(DateError::InvalidMonth())
        } else {
            Ok(Month(raw))
        }
    }
}
impl Month {
    pub fn inner(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_budget_csv_includes_multiple_items_in_same_category() {
        let category = Category {
            id: 1,
            name: "Groceries".to_string(),
            category_type: CategoryType::Expense,
        };
        let mut category_map = BTreeMap::new();
        category_map.insert(
            category,
            vec![
                LineItem {
                    id: 1,
                    description: "Milk".to_string(),
                    amount: 499,
                    category: 1,
                },
                LineItem {
                    id: 2,
                    description: "Bread".to_string(),
                    amount: 349,
                    category: 1,
                },
            ],
        );

        let mut budget = BTreeMap::new();
        budget.insert((2026, 1), category_map);

        let csv = FullBudgetResponse { budget }.as_csv();

        assert!(csv.contains("2026,1,Groceries,Milk,-4.99\n"));
        assert!(csv.contains("2026,1,Groceries,Bread,-3.49\n"));
    }
}
