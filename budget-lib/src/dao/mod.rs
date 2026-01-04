pub(crate) mod sqlite_dao;

use crate::types;

pub trait Dao {
    fn create_user(&self, req: &types::CreateUserRequest) -> Result<(), types::CreateUserError>;
    fn delete_user(&self, req: &types::DeleteUserRequest) -> Result<(), types::DeleteUserError>;
    fn delete_line_item(
        &self,
        req: &types::DeleteLineItemRequest,
    ) -> Result<(), types::DeleteLineItemError>;
    fn get_budget(
        &self,
        req: &types::GetBudgetRequest,
    ) -> Result<types::GetBudgetResponse, types::GetBudgetError>;
}
