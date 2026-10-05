//! Exercise 3: role-based access control.
//!
//! Code checks *permissions*, never role names: `if role == Admin` scattered
//! through handlers is impossible to audit or change. Roles are just named
//! bundles of permissions, defined in one place.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Moderator,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    ReadOwnProfile,
    ReadReports,
    ManageUsers,
}

impl Role {
    pub fn permissions(self) -> &'static [Permission] {
        use Permission::*;
        match self {
            Role::User => &[ReadOwnProfile],
            Role::Moderator => &[ReadOwnProfile, ReadReports],
            Role::Admin => &[ReadOwnProfile, ReadReports, ManageUsers],
        }
    }

    pub fn can(self, p: Permission) -> bool {
        self.permissions().contains(&p)
    }
}
