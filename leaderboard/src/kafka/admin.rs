use crate::state::team::{ConsumerGroupStatus, GroupState};
use anyhow::Result;
#[allow(unused_imports)]
use rdkafka::admin::{AdminClient, AdminOptions};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;

/// Create a Kafka admin client
pub fn create_admin_client(
    brokers: &str,
    username: &str,
    password: &str,
) -> Result<AdminClient<DefaultClientContext>> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", username)
        .set("sasl.password", password)
        .create()?;

    Ok(admin)
}

/// Fetch consumer group status for all teams (team-1 through team-15)
/// Note: This is a simplified implementation that just initializes empty statuses.
/// Full consumer group monitoring would require using list_groups which is more complex.
pub async fn fetch_consumer_group_statuses(
    _admin: &AdminClient<DefaultClientContext>,
) -> Vec<ConsumerGroupStatus> {
    // For now, return empty status for each team
    // In a production implementation, you would use the admin client to list and describe groups
    let mut statuses = Vec::new();

    for i in 1..=15 {
        let team_name = format!("team-{}", i);
        statuses.push(ConsumerGroupStatus {
            team_name,
            state: GroupState::Unknown,
            members: 0,
            lag: 0,
        });
    }

    statuses
}
