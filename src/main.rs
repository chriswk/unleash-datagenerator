use clap::Parser;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{ClientBuilder, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[clap(short, long, default_value_t = 1000)]
    pub count: u32,

    #[clap(short, long, default_value_t = 10)]
    pub strategies_per_feature: u32,

    #[clap(short, long, default_value = "development")]
    pub environment: String,

    #[clap(short, long, default_value_t = false)]
    pub print_to_shell: bool,

    #[clap(short, long, env, default_value = "http://localhost:4242")]
    pub unleash_url: String,

    #[clap(short, long, env)]
    pub api_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum FeatureType {
    Release,
    Operational,
    Experiment,
    Permission,
}

impl Distribution<FeatureType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FeatureType {
        match rng.gen_range(0..=3) {
            0 => FeatureType::Release,
            1 => FeatureType::Operational,
            2 => FeatureType::Experiment,
            3 => FeatureType::Permission,
            _ => FeatureType::Release,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub name: String,
    pub description: Option<String>,
    pub feature_type: FeatureType,
    pub impression_data: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Operator {
    NotIn,
    In,
    StrEndsWith,
    StrStartsWith,
    StrContains,
    NumEq,
    NumGt,
    NumGte,
    NumLt,
    NumLte,
    DateAfter,
    DateBefore,
    SemverEq,
    SemverLt,
    SemverGt,
    Unknown(String),
}

impl Distribution<Operator> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Operator {
        match rng.gen_range(0..=14) {
            0 => Operator::NotIn,
            1 => Operator::In,
            2 => Operator::StrEndsWith,
            3 => Operator::StrStartsWith,
            4 => Operator::StrContains,
            5 => Operator::NumEq,
            6 => Operator::NumGt,
            7 => Operator::NumGte,
            8 => Operator::NumLt,
            9 => Operator::NumLte,
            10 => Operator::DateAfter,
            11 => Operator::DateBefore,
            12 => Operator::SemverEq,
            13 => Operator::SemverLt,
            14 => Operator::SemverGt,
            _ => Operator::Unknown("What are you on about".into()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Constraint {
    pub context_name: String,
    pub operator: Operator,
    pub case_insensitive: bool,
    pub inverted: bool,
    pub values: Vec<String>,
    pub value: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Strategy {
    pub name: String,
    pub title: String,
    pub disabled: bool,
    pub sort_order: u32,
    pub constraints: Vec<Constraint>,
    pub parameters: HashMap<String, String>,
    pub segments: Vec<u32>,
}

fn features_url(base_url: &Url) -> Url {
    let mut feature_url = base_url.clone();
    feature_url
        .path_segments_mut()
        .unwrap()
        .push("api")
        .push("admin")
        .push("projects")
        .push("default")
        .push("features");
    feature_url
}

fn strategies_url(features_url: &Url, feature_name: &str, env: &str) -> Url {
    let mut update_url = features_url.clone();
    update_url
        .path_segments_mut()
        .expect("Could not modify strategy URL")
        .push(feature_name)
        .push("environments")
        .push(env)
        .push("strategies");
    update_url
}

async fn post_data_to(
    url: String,
    api_key: String,
    environment: String,
    features: Vec<Feature>,
    feature_strategies: HashMap<String, Vec<Strategy>>,
) {
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(&api_key).unwrap());
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("Couldn't build reqwest client");
    let unleash_url = Url::parse(&url).expect("Couldn't parse unleash url");
    let feature_url = features_url(&unleash_url);
    println!("Posting {} features to {}", features.len(), feature_url);
    for feature in features {
        client
            .post(feature_url.clone())
            .json(&feature)
            .send()
            .await
            .expect("Failed to send feature");
    }
    for (feature_name, strategies) in feature_strategies {
        let update_url = strategies_url(&feature_url, &feature_name, &environment);
        println!(
            "Posting {} strategies to {}",
            strategies.len(),
            update_url.clone()
        );
        for strategy in strategies {
            client
                .post(update_url.clone())
                .json(&strategy)
                .send()
                .await
                .expect("Failed to send strategy");
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut rng = rand::thread_rng();
    let features = (0..args.count)
        .into_iter()
        .map(|_| Feature {
            name: Ulid::new().to_string(),
            description: None,
            feature_type: rand::random(),
            impression_data: rand::random(),
        })
        .collect::<Vec<Feature>>();
    let feature_to_strategies: HashMap<String, Vec<Strategy>> = features
        .iter()
        .map(|f| {
            let strategies = (0..args.strategies_per_feature)
                .into_iter()
                .map(|s| {
                    let title = format!("strategy_{}_{}", f.name.clone(), s);
                    let mut parameters = HashMap::new();
                    let rollout = rng.gen_range(1..100);
                    parameters.insert("rollout".into(), format!("{rollout}"));
                    parameters.insert("stickiness".into(), "default".into());
                    parameters.insert("groupId".into(), f.name.clone());
                    Strategy {
                        name: "gradualRollout".into(),
                        title: title.clone(),
                        disabled: false,
                        sort_order: rng.gen_range(1..100000),
                        constraints: vec![],
                        parameters,
                        segments: vec![],
                    }
                })
                .collect::<Vec<Strategy>>();
            (f.name.clone(), strategies)
        })
        .collect::<HashMap<String, Vec<Strategy>>>();

    if args.print_to_shell {
        println!("{:?}", features);
        println!("{:?}", feature_to_strategies);
    } else if args.api_key.is_some() {
        post_data_to(
            args.unleash_url,
            args.api_key.unwrap_or("invalidkey".into()),
            args.environment,
            features,
            feature_to_strategies,
        )
        .await;
    }
}
