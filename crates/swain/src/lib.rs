use std::{future::Future, sync::Arc};

use self::error::Result;
use client::RiotHttpClient;
use error::Error;
use reqwest::Client;

pub mod client;
pub mod dto;
pub mod error;
pub mod rate_limit;
pub mod request;

#[derive(Clone)]
pub struct Swain {
    client: Arc<RiotHttpClient>,
}

impl Swain {
    pub fn new(user_agent: String, api_key: String) -> Self {
        let http_client = Client::builder().user_agent(user_agent).build().unwrap();
        let client = Arc::new(RiotHttpClient::new(http_client, api_key));
        Self { client }
    }

    pub fn set_api_key(&self, api_key: &str) {
        self.client.set_api_key(api_key);
    }

    pub fn api_key(&self) -> String {
        self.client.api_key()
    }

    pub async fn request<M>(&self, method: M) -> Result<M::Output>
    where
        M: 'static + Send + Sync + Method,
    {
        let mut remaining_attempts = 3;
        while remaining_attempts > 0 {
            remaining_attempts -= 1;
            let err = match method.request(&self.client).await {
                Ok(response) => return Ok(response),
                Err(Error::TooManyRequests) => Error::TooManyRequests,
                Err(Error::ApiError(api_err)) if api_err.status_code.is_server_error() => {
                    Error::ApiError(api_err)
                }
                Err(err) => return Err(err),
            };

            if remaining_attempts == 0 {
                return Err(err);
            }
        }
        Err(Error::TooManyAttempts)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RiotRegion {
    Americas,
    Asia,
    Europe,
    Esports,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LolRegion {
    Br,
    Eun,
    Euw,
    Jp,
    Kr,
    Lan,
    Las,
    Na,
    Oc,
    Ph,
    Ru,
    Sg,
    Th,
    Tr,
    Tw,
    Vn,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Subdomain {
    Americas,
    Asia,
    Europe,
    Esports,

    /// BR
    Br1,
    /// EUN
    Eun1,
    /// EUW
    Euw1,
    /// JP
    Jp1,
    /// KR
    Kr,
    /// LAN
    La1,
    /// LAS
    La2,
    /// NA
    Na1,
    /// OC
    Oc1,
    /// PH
    Ph2,
    /// RU
    Ru,
    /// SG
    Sg2,
    /// TH
    Th2,
    /// TR
    Tr1,
    /// TW
    Tw2,
    /// VN
    Vn2,
}

impl Subdomain {
    pub const VARIANTS: &'static [Self] = &[
        Self::Americas,
        Self::Asia,
        Self::Europe,
        Self::Esports,
        Self::Br1,
        Self::Eun1,
        Self::Euw1,
        Self::Jp1,
        Self::Kr,
        Self::La1,
        Self::La2,
        Self::Na1,
        Self::Oc1,
        Self::Ph2,
        Self::Ru,
        Self::Sg2,
        Self::Th2,
        Self::Tr1,
        Self::Tw2,
    ];

    pub(crate) fn domain(&self) -> &str {
        match self {
            Self::Americas => "americas.api.riotgames.com",
            Self::Asia => "asia.api.riotgames.com",
            Self::Europe => "europe.api.riotgames.com",
            Self::Esports => "esports.api.riotgames.com",
            Self::Br1 => "br1.api.riotgames.com",
            Self::Eun1 => "eun1.api.riotgames.com",
            Self::Euw1 => "euw1.api.riotgames.com",
            Self::Jp1 => "jp1.api.riotgames.com",
            Self::Kr => "kr.api.riotgames.com",
            Self::La1 => "la1.api.riotgames.com",
            Self::La2 => "la2.api.riotgames.com",
            Self::Na1 => "na1.api.riotgames.com",
            Self::Oc1 => "oc1.api.riotgames.com",
            Self::Ph2 => "ph2.api.riotgames.com",
            Self::Ru => "ru.api.riotgames.com",
            Self::Sg2 => "sg2.api.riotgames.com",
            Self::Th2 => "th2.api.riotgames.com",
            Self::Tr1 => "tr1.api.riotgames.com",
            Self::Tw2 => "tw2.api.riotgames.com",
            Self::Vn2 => "vn2.api.riotgames.com",
        }
    }
}

impl From<RiotRegion> for Subdomain {
    fn from(shard: RiotRegion) -> Self {
        match shard {
            RiotRegion::Americas => Subdomain::Americas,
            RiotRegion::Asia => Subdomain::Asia,
            RiotRegion::Europe => Subdomain::Europe,
            RiotRegion::Esports => Subdomain::Esports,
        }
    }
}

impl From<LolRegion> for Subdomain {
    fn from(shard: LolRegion) -> Self {
        match shard {
            LolRegion::Br => Subdomain::Br1,
            LolRegion::Eun => Subdomain::Eun1,
            LolRegion::Euw => Subdomain::Euw1,
            LolRegion::Jp => Subdomain::Jp1,
            LolRegion::Kr => Subdomain::Kr,
            LolRegion::Lan => Subdomain::La1,
            LolRegion::Las => Subdomain::La2,
            LolRegion::Na => Subdomain::Na1,
            LolRegion::Oc => Subdomain::Oc1,
            LolRegion::Ph => Subdomain::Ph2,
            LolRegion::Ru => Subdomain::Ru,
            LolRegion::Sg => Subdomain::Sg2,
            LolRegion::Th => Subdomain::Th2,
            LolRegion::Tr => Subdomain::Tr1,
            LolRegion::Tw => Subdomain::Tw2,
            LolRegion::Vn => Subdomain::Vn2,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum MethodId {
    AccountV1(AccountV1MethodId),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum AccountV1MethodId {
    GetAccountByRiotId,
}

pub trait Method {
    type Output;

    fn request(&self, client: &RiotHttpClient)
        -> impl Send + Future<Output = Result<Self::Output>>;
}
