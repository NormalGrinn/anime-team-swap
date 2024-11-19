use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
    pub(crate) team_id: u64,
    pub(crate) team_image_url: Option<String>,
    pub(crate) team_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Member{
    pub(crate) member_id: u64,
    pub(crate) member_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TeamMembers {
    pub(crate) team: Team,
    pub(crate) members: Vec<Member>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SubmittedAnime {
    pub(crate) anime_id: u64,
    pub(crate) anime_name: String,
    pub(crate) submitter_name: String,
    pub(crate) claimed_by_team: Option<String>,
    pub(crate) claimed_on: Option<String>,
}