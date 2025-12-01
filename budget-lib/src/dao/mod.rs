pub(crate) mod sqlite_dao;

use crate::types;

pub trait Dao {
    fn create_user(&self, req: &types::CreateUserRequest) -> Result<(), types::CreateUserError>;
}
