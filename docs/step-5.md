# Step 5: Keys and Log Compaction (Team Stats)

Your fraud detection pipeline is working! Now your manager wants visibility into each team's progress. You'll build a **real-time dashboard** that publishes your processing statistics.

Since teams may restart their consumers, the dashboard should always show the **latest** stats—not historical values. This is a perfect use case for **log compaction** with **message keys**.

---

## Understanding Message Keys

Until now, you've only sent message **values**—the JSON payload. But Kafka messages actually have two parts: a **key** and a **value**.

### Key vs Value

| Component | Purpose | Example |
|-----------|---------|---------|
| **Key** | Identifies the entity | `"team-1"` (your team name) |
| **Value** | Contains the data | `{"team": "team-1", "processed": 150, "flagged": 23}` |

### Why Use Keys?

Keys serve two important purposes:

1. **Partition Routing**: Messages with the same key always go to the same partition. This guarantees ordering for that key.

2. **Log Compaction**: When enabled, Kafka keeps only the **latest** value for each key—perfect for maintaining current state.

```
Without key:                                With key:
Messages go to random partitions            Same key → same partition
┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐         ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐
│ P0 │ │ P1 │ │ P2 │ │ P3 │ │ P4 │         │ P0 │ │ P1 │ │ P2 │ │ P3 │ │ P4 │
├────┤ ├────┤ ├────┤ ├────┤ ├────┤         ├────┤ ├────┤ ├────┤ ├────┤ ├────┤
│ A  │ │ B  │ │ A  │ │ C  │ │ B  │ ← rand  │ A  │ │ B  │ │ C  │ │ D  │ │ E  │
│ C  │ │ A  │ │ B  │ │ A  │ │ C  │         │ A  │ │ B  │ │ C  │ │ D  │ │ E  │
│ B  │ │ C  │ │ C  │ │ B  │ │ A  │         │ A  │ │ B  │ │ C  │ │ D  │ │ E  │
└────┘ └────┘ └────┘ └────┘ └────┘         └────┘ └────┘ └────┘ └────┘ └────┘
                                                 ↑ All "A" messages together
```

---

## Understanding Log Compaction

### Regular Topics vs Compacted Topics

Kafka topics can use different **retention policies**:

| Policy | Behavior | Use Case |
|--------|----------|----------|
| **Time-based** | Delete messages older than X hours/days | Event logs, audit trails |
| **Size-based** | Delete oldest messages when topic exceeds X GB | Bounded storage |
| **Compaction** | Keep only latest value per key | Current state, dashboards |

### How Compaction Works

In a compacted topic, Kafka periodically removes older records, keeping only the **most recent value for each unique key**:

```
BEFORE COMPACTION:                      AFTER COMPACTION:
┌────────────────────────────────┐      ┌────────────────────────────────┐
│ Key: "team-1"  │ flagged: 10   │      │                                │
│ Key: "team-2"  │ flagged: 5    │      │                                │
│ Key: "team-1"  │ flagged: 15   │  ──► │ Key: "team-2"  │ flagged: 8    │
│ Key: "team-2"  │ flagged: 8    │      │ Key: "team-1"  │ flagged: 23   │
│ Key: "team-1"  │ flagged: 23   │      │                                │
└────────────────────────────────┘      └────────────────────────────────┘
        5 messages                              2 messages
                                        (only latest per key!)
```

### Why Compaction is Powerful

1. **State Recovery**: On restart, read the entire compacted topic to rebuild your state. Since only the latest values exist, this is fast.

2. **Efficient Storage**: Old, superseded values are automatically cleaned up.

3. **Always Current**: Any new consumer reading from the beginning gets the current state, not historical values.

---

## New Topic: `team_stats`

The `team_stats` topic uses **log compaction**. This is perfect for our use case: tracking the *current* processing statistics per team, not the history of how they changed.

### Team Stats Record Schema

Each time you flag a user, produce a record like this:

```json
{
    "team": "team-1",
    "processed": 150,
    "flagged": 23
}
```

The **key** of the message must be your team name (e.g., `"team-1"`).

---

## TODOs

### 1. Track Stats in Memory

Add counters at the top of your code to track your processing:

```python
# Track processing statistics
processed_count = 0
flagged_count = 0
```

---

### 2. Update Your Processing Loop

Increment your counters as you process messages:

```python
# Inside your message processing loop:
processed_count += 1  # Count every message you see

# When your filter matches:
if your_filter_matches:
    flagged_count += 1
    
    # ... produce to actions topic (from Step 3) ...
    
    # Now produce stats update
    stats_record = {
        "team": TEAM_NAME,
        "processed": processed_count,
        "flagged": flagged_count
    }
    
    # The key MUST be your team name (as bytes)
    producer.send('team_stats',
                  key=TEAM_NAME.encode('utf-8'),
                  value=bytes(json.dumps(stats_record), 'utf-8'))
```

---

### 3. Produce to team_stats with KEY

The key difference from Step 3 is that you **must include a key**:

```python
producer.send('team_stats',
              key=TEAM_NAME.encode('utf-8'),           # <- KEY is required!
              value=bytes(json.dumps(stats_record), 'utf-8'))
```

**Important:** 
- The key must be encoded as bytes using `.encode('utf-8')`
- The key MUST match the `team` field in your JSON value
- The instructor's dashboard tracks whether you're using keys correctly!

---

## Check Your Work

| Leaderboard Shows | Meaning |
|-------------------|---------|
| 📊 in Progress | Success! Stats message received |
| 🔌 📤 ⚖️ 📊 all visible | Tutorial complete! |

The instructor's dashboard shows your live stats in a dedicated panel!

---

## Questions to Consider

1. What happens if you send a stats message without a key?
2. Why must the message key match the team name in the value?
3. What would a new consumer see if they read from the compacted topic from the beginning?

---

## Bonus Challenge: State Recovery

What happens if your program crashes and restarts? Your counters would reset to zero, and you'd lose track of previous progress!

Since the `team_stats` topic is compacted, you can rebuild your state by reading from it on startup. The compacted topic only keeps the latest value per key, so you'll quickly get the current stats.

**Hint:** Create a second consumer that reads from `team_stats` with `auto_offset_reset='earliest'` before starting your main processing loop. Filter for your team's key and populate your counters before processing new users.

```python
# On startup, restore state from compacted topic
restore_consumer = KafkaConsumer(
    'team_stats',
    bootstrap_servers=BOOTSTRAP_SERVERS,
    auto_offset_reset='earliest',
    consumer_timeout_ms=5000,  # Stop after 5 seconds of no messages
    # ... other auth settings ...
)

for message in restore_consumer:
    if message.key and message.key.decode('utf-8') == TEAM_NAME:
        data = json.loads(message.value.decode('utf-8'))
        processed_count = data['processed']
        flagged_count = data['flagged']
        print(f"Restored state: processed={processed_count}, flagged={flagged_count}")

restore_consumer.close()
```

This is an advanced challenge - ask your instructor for guidance if you get stuck!

---

## Congratulations!

You've completed the Kafka tutorial!

Throughout these 5 steps, you've learned:
- **Step 1**: How to consume messages from Kafka
- **Step 2**: How to filter and transform data
- **Step 3**: How to produce messages to Kafka
- **Step 4**: How to scale with Consumer Groups
- **Step 5**: How to use message keys and log compaction for stateful applications

These are the building blocks for real-world stream processing systems. You're now ready to tackle more advanced Kafka concepts like exactly-once semantics, Kafka Streams, and schema registries.

Happy streaming!
