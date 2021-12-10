use config::{ConfigError, File};
use fake::faker::{address, company, creditcard, internet, job, name};
use fake::Fake;
use log::info;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time;

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
}

impl User {
    pub fn new() -> Self {
        Self {
            email: internet::en::FreeEmail().fake(),
            user_name: internet::en::Username().fake(),
            user_agent: internet::en::UserAgent().fake(),
            avatar: format!(
                "https://robohash.org/{}.png?size=50x50",
                internet::en::Username().fake::<String>()
            ),
            field: job::en::Field().fake(),
            company_name: company::en::CompanyName().fake(),
            company_slogan: company::en::CatchPhase().fake(),
            profession: company::en::Profession().fake(),
            industry: company::en::Industry().fake(),
            premium: fake::faker::boolean::en::Boolean(50).fake(),
            credit: (-20..20).fake::<i32>(),
            time_zone: address::en::TimeZone().fake(),
            name: name::en::Name().fake(),
            credit_card_number: creditcard::en::CreditCardNumber().fake(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub interval: u64,
    pub topic: String,
    pub brokers: String,
    pub username: String,
    pub password: String,
}

// https://github.com/mehcode/config-rs/blob/master/examples/hierarchical-env/src/settings.rs#L39
impl Settings {
    pub fn from(path: &str) -> Result<Self, ConfigError> {
        let mut settings = config::Config::default();
        // Start off by merging in the "default" configuration file
        settings.merge(File::with_name(path).required(true))?;

        settings.try_into()
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

    let mut interval = time::interval(Duration::from_secs(settings.interval));

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
            FutureRecord::to(topic_name)
                .payload(&value)
                .key("")
                .headers(OwnedHeaders::new().add("source", "producer")),
            Duration::from_secs(0),
        )
        .await
    {
        Ok((partition, offset)) => log::trace!(
            "pushed message to partition {} at offset {}",
            partition,
            offset
        ),
        Err((error, message)) => log::error!("error pushing message: {:?}: {:?}", error, message),
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
