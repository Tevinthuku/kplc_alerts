use anyhow::anyhow;
use std::collections::HashSet;

pub struct Permissions {
    permissions: HashSet<Permission>,
}

impl Permissions {
    fn contains(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }
}

#[derive(Copy, Clone)]
pub enum Permission {
    ImportAffectedAreas,
}
pub trait Actor {
    fn permissions(&self) -> Permissions;

    fn check_for_permission(&self, permission: Permission) -> anyhow::Result<()> {
        match self.permissions().contains(permission) {
            true => Ok(()),
            false => {
                anyhow!("Unauthorized")
            }
        }
    }
}
