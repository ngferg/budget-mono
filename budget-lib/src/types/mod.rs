pub(crate) mod dao;
use std::collections;

#[derive(thiserror::Error, Debug)]
pub enum CreateUserError {
    #[error("Email improperly formatted")]
    EmailImproperlyFormatted(),
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
    pub email: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct DeleteUserRequest {
    pub email: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct DeleteLineItemRequest {
    pub item_id: u64,
    pub email: String,
    pub year: u32,
    pub month: u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct AddLineItemRequest {
    pub email: String,
    pub year: u32,
    pub month: u32,
    pub category_id: u64,
    pub description: String,
    pub amount: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct EditLineItemRequest {
    pub email: String,
    pub item_id: u64,
    pub description: String,
    pub amount: u64,
}

#[derive(Debug, serde::Deserialize)]
pub struct GetBudgetRequest {
    pub email: String,
    pub year: u32,
    pub month: u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct CloneMonthRequest {
    pub email: String,
    pub source_year: u32,
    pub source_month: u32,
    pub target_year: u32,
    pub target_month: u32,
}

#[derive(Debug, serde::Serialize)]
pub struct GetBudgetResponse {
    pub categories: Vec<Category>,
    pub budget: collections::HashMap<u64, Vec<LineItem>>,
    pub last_month_clonable: bool,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct VerifyTokenRequest {
    pub email: String,
    pub token: String,
}

#[derive(Debug, serde::Serialize)]
pub enum CategoryType {
    Income,
    Expense,
}

#[derive(Debug, serde::Serialize)]
pub struct Category {
    pub id: u64,
    pub name: String,
    pub category_type: CategoryType,
}

#[derive(Debug, serde::Serialize)]
pub struct LineItem {
    pub id: u64,
    pub description: String,
    pub amount: u64,
    pub category: u64,
}
