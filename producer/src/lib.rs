use config::{Config, ConfigError, File};
use fake::faker::{address, company, creditcard, internet, job, name};
use fake::Fake;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time;
use tracing::{error, info, trace};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    email: String,
    credit_card_number: String,
    company_name: String,
    company_slogan: String,
    industry: String,
    user_name: String,
    avatar: String,
    name: String,
    profession: String,
    field: String,
    premium: bool,
    credit: i32,
    time_zone: String,
    user_agent: String,
    pack: String,
}

impl User {
    pub fn new() -> Self {
        Self {
            email: internet::en::FreeEmail().fake(),
            user_name: internet::en::Username().fake(),
            user_agent: internet::en::UserAgent().fake(),
            avatar: if fake::faker::boolean::en::Boolean(90).fake() {
                format!(
                    "https://robohash.org/{}.png?size=50x50",
                    internet::en::Username().fake::<String>()
                )
            } else {
                String::from("example.org")
            },
            field: job::en::Field().fake(),
            company_name: company::en::CompanyName().fake(),
            company_slogan: company::en::CatchPhrase().fake(),
            profession: company::en::Profession().fake(),
            industry: company::en::Industry().fake(),
            premium: fake::faker::boolean::en::Boolean(50).fake(),
            credit: (-20..20).fake::<i32>(),
            time_zone: address::en::TimeZone().fake(),
            name: if fake::faker::boolean::en::Boolean(90).fake() {
                format!("{}", name::en::Name().fake::<String>())
            } else {
                String::from("John Doe")
            },
            credit_card_number: creditcard::en::CreditCardNumber().fake(),
            pack: if fake::faker::boolean::en::Boolean(10).fake() {
                String::from("free")
            } else {
                String::from("small")
            },
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub interval_millis: u64,
    pub topic: String,
    pub brokers: String,
    pub username: String,
    pub password: String,
}

// https://github.com/mehcode/config-rs/blob/master/examples/hierarchical-env/src/settings.rs#L39
impl Settings {
    pub fn from(path: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(path))
            .build()?;

        s.try_deserialize()
    }
}

pub async fn produce(settings: &Settings) {
    let producer: &FutureProducer = &ClientConfig::new()
        .set("client.id", "kafka-demo-producer")
        .set("bootstrap.servers", &settings.brokers)
        .set("message.timeout.ms", "5000")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", &settings.username)
        .set("sasl.password", &settings.password)
        .create()
        .expect("Producer creation error");

    let mut interval = time::interval(Duration::from_millis(settings.interval_millis));

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("received a <CTRL-C>, exiting");
                return;
            }
            _ = interval.tick() => {
                send_message(&producer, &settings.topic).await;
            }

        }
    }
}

async fn send_message(producer: &FutureProducer, topic_name: &str) {
    let user = User::new();
    let value = serde_json::to_string(&user).unwrap();
    match producer
        .send(
            FutureRecord::to(topic_name).payload(&value).key(""),
            Duration::from_secs(0),
        )
        .await
    {
        Ok((partition, offset)) => trace!(
            "pushed message to partition {} at offset {}",
            partition,
            offset
        ),
        Err((error, message)) => error!("error pushing message: {:?}: {:?}", error, message),
    };
}
#[cfg(test)]
mod tests {
    use crate::User;

    #[test]
    fn show_user() {
        dbg!(User::new());
    }
}
