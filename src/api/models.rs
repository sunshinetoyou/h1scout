use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramList {
    pub data: Vec<ProgramData>,
    pub links: Option<Links>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub attributes: ProgramAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAttributes {
    pub handle: String,
    pub name: String,
    pub offers_bounties: bool,
    pub submission_state: String,
    pub fast_payments: bool,
    pub open_scope: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeList {
    pub data: Vec<ScopeData>,
    pub links: Option<Links>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub attributes: ScopeAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeAttributes {
    pub asset_type: String,
    pub asset_identifier: String,
    pub eligible_for_bounty: bool,
    pub eligible_for_submission: bool,
    pub max_severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Links {
    pub next: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_program_list() {
        let json = include_str!("../../tests/fixtures/programs_page1.json");
        let result: ProgramList = serde_json::from_str(json).unwrap();
        assert_eq!(result.data.len(), 3);
        assert_eq!(result.data[0].attributes.handle, "general-motors");
        assert!(result.data[0].attributes.offers_bounties);
        assert!(result.links.as_ref().unwrap().next.is_some());
    }

    #[test]
    fn test_parse_scope_list() {
        let json = include_str!("../../tests/fixtures/scopes_android.json");
        let result: ScopeList = serde_json::from_str(json).unwrap();
        assert_eq!(result.data.len(), 3);
        assert_eq!(result.data[0].attributes.asset_type, "ANDROID");
    }
}
