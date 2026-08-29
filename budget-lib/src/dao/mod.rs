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
    fn get_subscription(
        &self,
        req: &types::GetSubscriptionRequest,
    ) -> Result<types::Subscription, types::GetSubscriptionError>;
    /// Marks the subscription `active` and records the Stripe identifiers from a
    /// completed Checkout Session. A `lifetime` row is left untouched.
    fn activate_subscription(
        &self,
        req: &types::ActivateSubscriptionRequest,
    ) -> Result<(), types::RecordCheckoutError>;
    /// Applies a Stripe subscription-lifecycle change (a cancel, a renewal, or
    /// the subscription ending) to the row, but only when the stored
    /// `stripe_subscription_id` matches. A `lifetime` row is left untouched.
    fn sync_subscription(
        &self,
        req: &types::SyncSubscriptionRequest,
    ) -> Result<(), types::SyncSubscriptionError>;
}
