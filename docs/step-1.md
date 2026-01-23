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
With your environment set up (see Step 0), the boilerplate code in `main.py` is ready for you to begin. 

1. Replace `TEAM_NAME` with your assigned team name (e.g., `team-2`, `team-3`, etc.). The other credentials are auto-configured.
2. Run the code. If everything is configured correctly, you should see the Kafka messages being displayed in the output.

#### Questions:

- How many partitions does the topic `new_users` have?

> **Hint:** Look at the consumer output format: `topic:partition:offset`. The partition number is the second value.

---

## Understanding Topics and Partitions

When you run your consumer, you'll see output like `new_users:1:42`. Let's break down what this means.

### What is a Topic?

A **topic** is a named stream of messages—think of it like a database table or a message channel. In our case, `new_users` is a topic where all new user registration events are published.

- Topics are **logical categories** for organizing messages
- Producers write to topics, consumers read from them
- Multiple producers can write to the same topic
- Multiple consumers can read from the same topic independently

### What is a Partition?

Each topic is split into one or more **partitions**. A partition is an ordered, immutable sequence of messages—essentially an append-only log file.

```
Topic: new_users
┌───────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│  Partition 0    Partition 1    Partition 2    Partition 3    Partition 4             │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐            │
│  │ offset 0 │   │ offset 0 │   │ offset 0 │   │ offset 0 │   │ offset 0 │            │
│  │ offset 1 │   │ offset 1 │   │ offset 1 │   │ offset 1 │   │ offset 1 │            │
│  │ offset 2 │   │ offset 2 │   │ offset 2 │   │ offset 2 │   │ offset 2 │            │
│  │   ...    │   │   ...    │   │   ...    │   │   ...    │   │   ...    │            │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘   └──────────┘            │
│       ▲              ▲              ▲              ▲              ▲                   │
│   append here    append here   append here    append here    append here             │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

**Why partitions?**
- **Parallelism**: Different partitions can be consumed in parallel by different consumers
- **Scalability**: Data is distributed across multiple servers
- **Throughput**: More partitions = more concurrent readers/writers

### What is an Offset?

An **offset** is a unique identifier for each message within a partition—like a line number in a file. Offsets are:
- **Sequential**: Start at 0 and increment by 1 for each new message
- **Immutable**: Once assigned, an offset never changes
- **Partition-specific**: Offset 42 in partition 0 is different from offset 42 in partition 1

So when you see `new_users:1:42`, it means:
- **Topic**: `new_users`
- **Partition**: `1`
- **Offset**: `42` (the 43rd message in that partition)

### Ordering Guarantees

Messages are **strictly ordered within a partition**, but there's **no ordering guarantee across partitions**. This means:
- Messages in partition 0 are always read in the order they were written
- But a message in partition 1 might be read before or after a message in partition 0, even if it was produced later

This is why partition count matters for your application design—and why we'll explore Consumer Groups in Step 4.

---

## Check Your Work

| Leaderboard Shows | Meaning |
|-------------------|---------|
| 🔌 in Progress | Success! You're connected |
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

### Understanding Bytes vs Strings

Why do we need `.decode('utf-8')`? Let's break it down:

```python
# Kafka transmits raw bytes over the network, not text
raw_bytes = message.value          # b'{"email": "alice@example.com"}'

# .decode() converts bytes to a Python string using UTF-8 encoding
json_string = raw_bytes.decode('utf-8')  # '{"email": "alice@example.com"}'

# json.loads() converts the JSON string to a Python dictionary
user = json.loads(json_string)     # {"email": "alice@example.com"}

# Now you can access fields
print(user["email"])               # alice@example.com
```

**Why bytes?** Kafka is language-agnostic—it doesn't know if you're using Python, Java, or Go. Bytes are the universal format that all languages understand. UTF-8 is the standard encoding that maps characters (like `a`, `é`, `日`) to bytes.

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
