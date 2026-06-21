pub(crate) mod sqlite_dao;

use std::collections::HashMap;

use crate::types;

pub trait Dao {
    fn create_user(&self, req: &types::CreateUserRequest) -> Result<(), types::CreateUserError>;
    fn delete_user(&self, req: &types::DeleteUserRequest) -> Result<(), types::DeleteUserError>;
    fn add_line_item(&self, req: &types::AddLineItemRequest)
    -> Result<(), types::AddLineItemError>;
    fn edit_line_item(
        &self,
        req: &types::EditLineItemRequest,
    ) -> Result<(), types::EditLineItemError>;
    fn delete_line_item(
        &self,
        req: &types::DeleteLineItemRequest,
    ) -> Result<(), types::DeleteLineItemError>;
    fn get_all_categories(
        &self,
        req: &types::GetAllCategoriesRequest,
    ) -> Result<HashMap<u64, types::Category>, types::GetBudgetError>;
    fn get_all_line_items(
        &self,
        req: &types::GetAllLineItemsRequest,
    ) -> Result<Vec<types::FullLineItem>, types::GetBudgetError>;
    fn get_budget(
        &self,
        req: &types::GetBudgetRequest,
    ) -> Result<types::GetBudgetResponse, types::GetBudgetError>;
    fn clone_month(&self, req: &types::CloneMonthRequest) -> Result<(), types::CloneMonthError>;
    fn add_category(&self, req: &types::AddCategoryRequest) -> Result<(), types::AddCategoryError>;
}
