use std::fmt::{Display, Formatter};

use time::OffsetDateTime;

pub type Timestamp = OffsetDateTime;

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
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

string_id!(TaskId);
string_id!(TaskSlug);
string_id!(RepoId);
string_id!(RepoGroupId);
string_id!(SkillId);
string_id!(SkillSourceId);
string_id!(SkillVersion);
string_id!(MountId);
string_id!(KnowledgeId);
string_id!(KnowledgeCandidateId);
