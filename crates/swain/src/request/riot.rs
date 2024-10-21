use crate::{
    client::RiotHttpClient, dto::AccountDto, error::Result, AccountV1MethodId, Method, MethodId,
    RiotRegion,
};

#[derive(Debug)]
pub struct GetAccountByRiotId {
    region: RiotRegion,
    game_name: String,
    tag_line: String,
}

impl GetAccountByRiotId {
    pub fn new(region: RiotRegion, game_name: String, tag_line: String) -> Self {
        Self {
            region,
            game_name,
            tag_line,
        }
    }
}

impl Method for GetAccountByRiotId {
    type Output = AccountDto;

    async fn request(&self, client: &RiotHttpClient) -> Result<Self::Output> {
        let path = format!(
            "/riot/account/v1/accounts/by-riot-id/{}/{}",
            self.game_name, self.tag_line
        );
        let method_id = MethodId::AccountV1(AccountV1MethodId::GetAccountByRiotId);
        let request = client.get(&path, method_id, self.region.into());
        request.send_riot_json().await
    }
}
