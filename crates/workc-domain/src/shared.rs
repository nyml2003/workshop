use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub type Timestamp = OffsetDateTime;

macro_rules! nanoid_id {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn generate() -> Self {
                Self(nanoid::nanoid!(8))
            }

            pub fn from_string(value: &str) -> Self {
                Self(value.to_owned())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

nanoid_id!(TaskSlug);
nanoid_id!(RepoId);
nanoid_id!(RepoGroupId);
nanoid_id!(SkillId);
nanoid_id!(SkillSourceId);
nanoid_id!(SkillVersion);
nanoid_id!(MountId);
nanoid_id!(KnowledgeId);
nanoid_id!(KnowledgeCandidateId);
