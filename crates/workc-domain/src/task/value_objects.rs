#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Draft,
    Active,
    Closed,
    Archived,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskSkillMountStatus {
    Active,
    Inactive,
    Removed,
}
