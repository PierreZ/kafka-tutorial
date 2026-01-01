# Step 1: Extract

The very first step to do is to connect to Kafka and display all incoming messages. Let's do this!

## Topics to read

### `new_users`

The main topic is named "new_users". Here's an example of the JSON pushed:

```json
{
    "email": "hugh_atque@hotmail.com",
    "credit_card_number": "373947589498776",
    "company_name": "Breitenberg and Sons",
    "company_slogan": "Open-architected directional adapter",
    "industry": "Market Research",
    "user_name": "ilene_quaerat",
    "avatar": "https://robohash.org/marcus_omnis.png?size=50x50",
    "name": "Roslyn Dicki",
    "profession": "advocate",
    "field": "Mining",
    "premium": true,
    "credit": -7,
    "time_zone": "Pacific/Pago_Pago",
    "user_agent": "Mozilla/5.0 (Windows; U; MSIE 9.0; WIndows NT 9.0; en-US))",
    "pack": "small"
}
```

## TODOs

### Step 1: Read Kafka Messages
With the provided GitPod setup, the boilerplate code is already in place for you to begin. 

1. Replace the placeholders `BOOTSTRAP_SERVERS`, `TEAM_NAME`, `SASL_USERNAME`, and `SASL_PASSWORD` with the appropriate values in the code.
2. Run the code. If everything is configured correctly, you should see the Kafka messages being displayed in the output.

#### Questions:

- How many partitions does the topic `new_users` have?

> **Hint:** Look at the consumer output format: `topic:partition:offset`. The partition number is the second value.

---

## Understanding Partitions

A **partition** is Kafka's unit of parallelism. Each topic is split into one or more partitions:

- **Ordering**: Messages within a partition are strictly ordered
- **Parallelism**: Different partitions can be consumed in parallel by different consumers
- **Distribution**: In Step 4, you'll see how Consumer Groups distribute partitions among team members

With multiple partitions, team members can each consume from different partitions simultaneously in Step 4.

---

## Check Your Work

| Leaderboard Shows | Meaning |
|-------------------|---------|
| 1️⃣ in Progress | Success! You're connected |
| Team stays ⬜ | Connection issue - see Troubleshooting |

> **⚔️ First Blood**: The first team to connect wins this achievement! Speed matters.

### Step 2: Parsing the JSON
Once you see the full Kafka message displayed, it's time to interpret its contents. Each message contains a JSON payload, which can be parsed in Python using the `json` package.

1. Use the `json.loads` function to parse the message value.
2. To confirm successful parsing, extract and print only the `email` field from the JSON.

Example:
```python
import json

# message.value is bytes, so we need to decode it first
parsed_message = json.loads(message.value.decode('utf-8'))
print(parsed_message["email"])
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Connection Refused | Verify Kafka is running (ask instructor); check `bootstrap_servers` matches provided address |
| Authentication Failed | Username must be lowercase (e.g., `team-1`, not `Team-1`); verify password with instructor |
| No Messages Appearing | Verify topic is exactly `new_users`; check `group_id` matches team name; wait a few seconds |

---

## Next step

Congratulations, you learned how to consume a message in Kafka 🎉
You can now continue on [step-2](/kafka-tutorial/docs/step-2.html)!
