use serde::{self, Deserialize, Deserializer};
use starknet::core::types::FieldElement;
use std::env;
use std::fs;

pub_struct!(Clone, Deserialize; Watchtower {
    enabled : bool,
    endpoint: String,
    app_id: String,
    token: String,
    types: WatchtowerTypes,
});

pub_struct!(Clone, Deserialize; WatchtowerTypes {
    info: String,
    warning: String,
    severe: String,
});

pub_struct!(Clone, Deserialize; Server { port: u16 });

pub_struct!(Clone, Deserialize; Database {
    name: String,
    connection_string: String,
});

pub_struct!(Clone, Deserialize; NftContract {
    address: String,
    private_key: FieldElement,
});

pub_struct!(Clone, Deserialize;  Variables {
    app_link: String,
    api_link: String,
    is_testnet: bool,
    proxy: Option<String>,
    rpc_url: String,
});

pub_struct!(Clone, Deserialize; StarknetIdContracts {
    naming_contract: FieldElement,
    verifier_contracts: Vec<FieldElement>,
    identity_contract: FieldElement,
});

pub_struct!(Clone, Deserialize;  Nostra {
    utils_contract: FieldElement,
    pairs : Vec<FieldElement>,
    staking_contract: FieldElement,
});

pub_struct!(Clone, Deserialize;  Pairs {
    utils_contract: FieldElement,
    pairs : Vec<FieldElement>,
});

pub_struct!(Clone, Deserialize;  Contract {
    contract: FieldElement,
});

pub_struct!(Clone, Deserialize;  TokenAndContract {
    token_address: FieldElement,
    contract: FieldElement,
});

pub_struct!(Clone, Deserialize;  Braavos {
    api_key_user: String,
    api_key_claimed_mission: String,
});

pub_struct!(Clone, Deserialize;  Element {
    api_key: String,
});

pub_struct!(Clone, Deserialize;  ZkLend {
    contract: FieldElement,
    utils_contract: FieldElement,
    pairs : Vec<FieldElement>,
});

pub_struct!(Clone, Deserialize;  Quests {
    utils_contract: FieldElement,
    sithswap: Pairs,
    zklend: ZkLend,
    jediswap: Pairs,
    ekubo: Contract,
    myswap: Contract,
    braavos: Braavos,
    element: Element,
    nostra: Nostra,
    carbonable: Contract,
    hashstack: TokenAndContract,
    haiko: PublicApi,
    nimbora: Contract,
    bountive: Contract,
    sithswap_2: ApiEndpoint,
});

pub_struct!(Clone, Deserialize;  Twitter {
    oauth2_clientid: String,
    oauth2_secret: String,
});

pub_struct!(Clone, Deserialize;  QuestBoost{
    private_key: FieldElement,
    update_interval: u64,
});

pub_struct!(Clone, Deserialize;  Discord {
    oauth2_clientid: String,
    oauth2_secret: String,
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuizQuestionType {
    TextChoice,
    ImageChoice,
    Ordering,
}

impl<'de> Deserialize<'de> for QuizQuestionType {
    fn deserialize<D>(deserializer: D) -> Result<QuizQuestionType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "text_choice" => Ok(QuizQuestionType::TextChoice),
            "image_choice" => Ok(QuizQuestionType::ImageChoice),
            "ordering" => Ok(QuizQuestionType::Ordering),
            _ => Err(serde::de::Error::custom("Unexpected type")),
        }
    }
}

pub_struct!(Clone, Deserialize,Debug; QuizQuestion {
    kind: QuizQuestionType,
    layout: String,
    question: String,
    options: Vec<String>,
    correct_answers: Option<Vec<usize>>,
    correct_order: Option<Vec<String>>,
    image_for_layout: Option<String>,
});

pub_struct!(Clone, Deserialize,Debug; Quiz {
    name: String,
    desc: String,
    questions: Vec<QuizQuestion>,
});

pub_struct!(Clone, Deserialize;  Starkscan {
    api_key: String,
});

pub_struct!(Clone, Deserialize; PublicApi  {
    api_endpoint: String,
});

pub_struct!(Clone, Deserialize;  Api {
    api_endpoint: String,
    api_key: String,
});

pub_struct!(Clone, Deserialize;  ApiEndpoint {
    api_endpoint: String,
});

pub_struct!(Clone, Deserialize;  Achievement {
    contract: FieldElement,
});

pub_struct!(Clone, Deserialize;  Achievements {
    braavos: Achievement,
    argent: Achievement,
    carbonable: Achievement,
});

pub_struct!(Clone, Deserialize;  AuthSetup {
    secret_key: String,
    expiry_duration: i64,
});

pub_struct!(Clone, Deserialize;  ProtocolStats {
    pairs_api_endpoint: String,
    lending_api_endpoint: String,
    derivates_api_endpoint: String,
    alt_protocols_api_endpoint: String,
});

pub_struct!(Clone, Deserialize;  Rewards {
    nimbora: Contract,
    vesu: Contract,
});

pub_struct!(Clone, Deserialize;  Token {
    contract: FieldElement,
    symbol: String,
    decimals: i64
});

pub_struct!(Clone, Deserialize;  Tokens {
    strk: Token,
});

pub_struct!(Clone, Deserialize;  Config {
    server: Server,
    database: Database,
    nft_contract: NftContract,
    discover:ProtocolStats,
    variables: Variables,
    starknetid_contracts: StarknetIdContracts,
    quests: Quests,
    twitter: Twitter,
    discord: Discord,
    starkscan: Starkscan,
    achievements: Achievements,
    watchtower: Watchtower,
    quest_boost: QuestBoost,
    rhino: PublicApi,
    rango: Api,
    pyramid: ApiEndpoint,
    auth:AuthSetup,
    rewards: Rewards,
    tokens: Tokens
});

pub fn load() -> Config {
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() <= 1 {
        "config.toml"
    } else if (args.len() > 1) && (args.get(1).unwrap().contains("toml")) {
        args.get(1).unwrap()
    } else {
        "config.toml"
    };
    let file_contents = fs::read_to_string(config_path);
    if file_contents.is_err() {
        panic!("error: unable to read file with path \"{}\"", config_path);
    }

    match toml::from_str(file_contents.unwrap().as_str()) {
        Ok(loaded) => loaded,
        Err(err) => {
            panic!("error: unable to deserialize config. {}", err);
        }
    }
}
