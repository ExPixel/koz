use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}
