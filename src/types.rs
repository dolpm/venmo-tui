use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    display_name: String,
    id: String,
    username: String,
    first_name: String,
    last_name: String,
    profile_picture_url: String,
    friend_count: u32,
    initials: String,
    friend_status: Option<bool>,
    is_blocked: bool,
    is_active: bool,
    identity_type: IdentityType,
    email: String,
    phone: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum IdentityType {
    Personal,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserBalance {
    pub value: f32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub user_balance: UserBalance,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Avatar {
    url: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    is_denylisted: bool,
    is_suspended: bool,
    #[serde(rename = "type")]
    account_type: IdentityType,
    avatar: Avatar,
    pub display_name: String,
    pub handle: String,
    pub id: String,
    pub balance: Balance,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StoryType {
    Payment,
    Transfer,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoryNote {
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SenderReciever {
    id: String,
    pub display_name: String,
    pub username: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum StorySubType {
    #[serde(rename = "p2p")]
    P2p,
    StandardTransfer,
    CreditReward,
    CreditRepayment,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoryPayload {
    pub sub_type: StorySubType,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoryTitle {
    pub payload: StoryPayload,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    pub receiver: Option<SenderReciever>,
    #[serde(default)]
    pub sender: Option<SenderReciever>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Story {
    pub amount: String,
    avatar: String,
    initials: String,
    pub date: String,
    id: String,
    pub note: StoryNote,
    pub title: StoryTitle,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoriesResponse {
    pub next_id: String,
    pub stories: Vec<Story>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PayRequestResponseStatus {
    Pending,
    Settled,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PayRequestResponse {
    pub balance: String,
    pub status: PayRequestResponseStatus,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundingInstrument {
    pub id: String,
    pub name: String,
    pub instrument_type: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Eligibility {
    pub eligibile: bool,
    pub eligibility_token: Option<String>,
}
