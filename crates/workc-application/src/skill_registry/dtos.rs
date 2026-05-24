use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplicationSkillSourceKind {
    Git,
    Local,
    Archive,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSkillSourceCommand {
    pub source_id: String,
    pub kind: ApplicationSkillSourceKind,
    pub location: String,
    pub reference: Option<String>,
    pub skills: Vec<ImportedSkillDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShowSkillQuery {
    pub skill_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedSkillDefinition {
    pub id: String,
    pub versions: Vec<String>,
    pub latest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SkillSummary {
    pub id: String,
    pub source: String,
    pub versions: Vec<String>,
    pub latest: Option<String>,
}
