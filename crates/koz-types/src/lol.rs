use std::str::FromStr;

use derive_more::derive::Display;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE", type_name = "lol_ranked_queue")]
pub enum LolRankedQueue {
    #[default]
    Solo,
    Flex,
    TwistedTreeline,
}

impl LolRankedQueue {
    pub const VARIANTS: [LolRankedQueue; 3] = [
        LolRankedQueue::Solo,
        LolRankedQueue::Flex,
        LolRankedQueue::TwistedTreeline,
    ];

    pub fn as_str_kebab(self) -> &'static str {
        match self {
            LolRankedQueue::Solo => "solo",
            LolRankedQueue::Flex => "flex",
            LolRankedQueue::TwistedTreeline => "twisted-treeline",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LolRankedQueue::Solo => "Solo",
            LolRankedQueue::Flex => "Flex",
            LolRankedQueue::TwistedTreeline => "Twisted Treeline",
        }
    }
}

impl std::fmt::Display for LolRankedQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.pad(self.as_str_kebab())
        } else {
            f.pad(self.as_str())
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE", type_name = "lol_region")]
pub enum LolRegion {
    Br,
    Eun,
    Euw,
    Jp,
    Kr,
    Lan,
    Las,
    #[default]
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

impl LolRegion {
    pub const VARIANTS: [LolRegion; 16] = [
        LolRegion::Br,
        LolRegion::Eun,
        LolRegion::Euw,
        LolRegion::Jp,
        LolRegion::Kr,
        LolRegion::Lan,
        LolRegion::Las,
        LolRegion::Na,
        LolRegion::Oc,
        LolRegion::Ph,
        LolRegion::Ru,
        LolRegion::Sg,
        LolRegion::Th,
        LolRegion::Tr,
        LolRegion::Tw,
        LolRegion::Vn,
    ];

    pub const AMERICAS: [LolRegion; 4] =
        [LolRegion::Na, LolRegion::Br, LolRegion::Lan, LolRegion::Las];
    pub const ASIA: [LolRegion; 9] = [
        LolRegion::Jp,
        LolRegion::Kr,
        LolRegion::Tw,
        LolRegion::Th,
        LolRegion::Sg,
        LolRegion::Vn,
        LolRegion::Tr,
        LolRegion::Ph,
        LolRegion::Oc, // idk
    ];
    pub const EUROPE: [LolRegion; 3] = [LolRegion::Eun, LolRegion::Euw, LolRegion::Ru];

    pub fn as_str_lower<'s>(self) -> &'s str {
        match self {
            LolRegion::Br => "br",
            LolRegion::Eun => "eun",
            LolRegion::Euw => "euw",
            LolRegion::Jp => "jp",
            LolRegion::Kr => "kr",
            LolRegion::Lan => "lan",
            LolRegion::Las => "las",
            LolRegion::Na => "na",
            LolRegion::Oc => "oc",
            LolRegion::Ph => "ph",
            LolRegion::Ru => "ru",
            LolRegion::Sg => "sg",
            LolRegion::Th => "th",
            LolRegion::Tr => "tr",
            LolRegion::Tw => "tw",
            LolRegion::Vn => "vn",
        }
    }

    pub fn as_str_upper<'s>(self) -> &'s str {
        match self {
            LolRegion::Br => "BR",
            LolRegion::Eun => "EUN",
            LolRegion::Euw => "EUW",
            LolRegion::Jp => "JP",
            LolRegion::Kr => "KR",
            LolRegion::Lan => "LAN",
            LolRegion::Las => "LAS",
            LolRegion::Na => "NA",
            LolRegion::Oc => "OC",
            LolRegion::Ph => "PH",
            LolRegion::Ru => "RU",
            LolRegion::Sg => "SG",
            LolRegion::Th => "TH",
            LolRegion::Tr => "TR",
            LolRegion::Tw => "TW",
            LolRegion::Vn => "VN",
        }
    }
}

impl FromStr for LolRegion {
    type Err = InvalidLolRegion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LolRegion::VARIANTS
            .into_iter()
            .find(|r| r.as_str_upper().eq_ignore_ascii_case(s))
            .ok_or(InvalidLolRegion)
    }
}

impl std::fmt::Display for LolRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.pad(self.as_str_lower())
        } else {
            f.pad(self.as_str_upper())
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("invalid lol region")]
pub struct InvalidLolRegion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, PartialOrd, Ord)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE", type_name = "lol_tier")]
pub enum LolTier {
    Iron,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Emerald,
    Diamond,
    Master,
    Grandmaster,
    Challenger,
}

impl LolTier {
    const VARIANTS: [LolTier; 10] = [
        LolTier::Iron,
        LolTier::Bronze,
        LolTier::Silver,
        LolTier::Gold,
        LolTier::Platinum,
        LolTier::Emerald,
        LolTier::Diamond,
        LolTier::Master,
        LolTier::Grandmaster,
        LolTier::Challenger,
    ];

    pub fn as_str_lower(&self) -> &'static str {
        match self {
            Self::Iron => "iron",
            Self::Bronze => "bronze",
            Self::Silver => "silver",
            Self::Gold => "gold",
            Self::Platinum => "platinum",
            Self::Emerald => "emerald",
            Self::Diamond => "diamond",
            Self::Master => "master",
            Self::Grandmaster => "grandmaster",
            Self::Challenger => "challenger",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Iron => "Iron",
            Self::Bronze => "Bronze",
            Self::Silver => "Silver",
            Self::Gold => "Gold",
            Self::Platinum => "Platinum",
            Self::Emerald => "Emerald",
            Self::Diamond => "Diamond",
            Self::Master => "Master",
            Self::Grandmaster => "Grandmaster",
            Self::Challenger => "Challenger",
        }
    }

    pub fn is_apex(self) -> bool {
        matches!(
            self,
            LolTier::Master | LolTier::Grandmaster | LolTier::Challenger
        )
    }
}

impl std::fmt::Display for LolTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.pad(self.as_str_lower())
        } else {
            f.pad(self.as_str())
        }
    }
}

impl std::str::FromStr for LolTier {
    type Err = InvalidLolTier;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for tier in Self::VARIANTS {
            if tier.as_str().eq_ignore_ascii_case(s) {
                return Ok(tier);
            }
        }
        Err(InvalidLolTier(s.to_owned()))
    }
}

#[derive(Debug, thiserror::Error, Clone)]
#[error("invalid tier: {0}")]
pub struct InvalidLolTier(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, Display)]
#[sqlx(transparent)]
pub struct LolDivision(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
#[display("{tier} {division}")]
pub struct LolRank {
    pub tier: LolTier,
    pub division: LolDivision,
}

impl LolRank {
    pub const fn parts(self) -> (LolTier, LolDivision) {
        (self.tier, self.division)
    }
}

impl LolRank {
    pub const ALL: [LolRank; 31] = [
        LolRank::new(LolTier::Iron, LolDivision(4)),
        LolRank::new(LolTier::Iron, LolDivision(3)),
        LolRank::new(LolTier::Iron, LolDivision(2)),
        LolRank::new(LolTier::Iron, LolDivision(1)),
        LolRank::new(LolTier::Bronze, LolDivision(4)),
        LolRank::new(LolTier::Bronze, LolDivision(3)),
        LolRank::new(LolTier::Bronze, LolDivision(2)),
        LolRank::new(LolTier::Bronze, LolDivision(1)),
        LolRank::new(LolTier::Silver, LolDivision(4)),
        LolRank::new(LolTier::Silver, LolDivision(3)),
        LolRank::new(LolTier::Silver, LolDivision(2)),
        LolRank::new(LolTier::Silver, LolDivision(1)),
        LolRank::new(LolTier::Gold, LolDivision(4)),
        LolRank::new(LolTier::Gold, LolDivision(3)),
        LolRank::new(LolTier::Gold, LolDivision(2)),
        LolRank::new(LolTier::Gold, LolDivision(1)),
        LolRank::new(LolTier::Platinum, LolDivision(4)),
        LolRank::new(LolTier::Platinum, LolDivision(3)),
        LolRank::new(LolTier::Platinum, LolDivision(2)),
        LolRank::new(LolTier::Platinum, LolDivision(1)),
        LolRank::new(LolTier::Emerald, LolDivision(4)),
        LolRank::new(LolTier::Emerald, LolDivision(3)),
        LolRank::new(LolTier::Emerald, LolDivision(2)),
        LolRank::new(LolTier::Emerald, LolDivision(1)),
        LolRank::new(LolTier::Diamond, LolDivision(4)),
        LolRank::new(LolTier::Diamond, LolDivision(3)),
        LolRank::new(LolTier::Diamond, LolDivision(2)),
        LolRank::new(LolTier::Diamond, LolDivision(1)),
        LolRank::new(LolTier::Master, LolDivision(1)),
        LolRank::new(LolTier::Grandmaster, LolDivision(1)),
        LolRank::new(LolTier::Challenger, LolDivision(1)),
    ];

    pub const fn new(tier: LolTier, division: LolDivision) -> Self {
        Self { tier, division }
    }
}
