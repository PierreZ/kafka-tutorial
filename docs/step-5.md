# Step 5: Stateful Processing (Watchlist)

Your fraud detection system from the previous steps is working well! But your mentor has a new observation: sometimes multiple flagged users come from the **same company**. When a company has 3 or more flagged users, it's worth investigating further.

Your task is to build a **watchlist** that tracks companies with suspicious activity.

---

## Understanding Message Keys

Until now, you've only sent message **values**—the JSON payload. But Kafka messages actually have two parts: a **key** and a **value**.

### Key vs Value

| Component | Purpose | Example |
|-----------|---------|---------|
| **Key** | Identifies the entity | `"Acme Corp"` (company name) |
| **Value** | Contains the data | `{"team": "team-1", "flag_count": 3}` |

### Why Use Keys?

Keys serve two important purposes:

1. **Partition Routing**: Messages with the same key always go to the same partition. This guarantees ordering for that key.

2. **Log Compaction**: When enabled, Kafka keeps only the **latest** value for each key—perfect for maintaining current state.

```
Without key:                      With key:
Messages go to random partitions  Same key → same partition
┌────┐ ┌────┐ ┌────┐              ┌────┐ ┌────┐ ┌────┐
│ P0 │ │ P1 │ │ P2 │              │ P0 │ │ P1 │ │ P2 │
├────┤ ├────┤ ├────┤              ├────┤ ├────┤ ├────┤
│ A  │ │ B  │ │ A  │  ← scattered │ A  │ │ B  │ │ C  │
│ C  │ │ A  │ │ B  │              │ A  │ │ B  │ │ C  │
│ B  │ │ C  │ │ C  │              │ A  │ │ B  │ │ C  │
└────┘ └────┘ └────┘              └────┘ └────┘ └────┘
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
| **Compaction** | Keep only latest value per key | Current state, snapshots |

### How Compaction Works

In a compacted topic, Kafka periodically removes older records, keeping only the **most recent value for each unique key**:

```
BEFORE COMPACTION:                      AFTER COMPACTION:
┌────────────────────────────────┐      ┌────────────────────────────────┐
│ Key: "Acme Corp"  │ count: 1   │      │                                │
│ Key: "Beta Inc"   │ count: 1   │      │                                │
│ Key: "Acme Corp"  │ count: 2   │  ──► │ Key: "Beta Inc"   │ count: 2   │
│ Key: "Beta Inc"   │ count: 2   │      │ Key: "Acme Corp"  │ count: 3   │
│ Key: "Acme Corp"  │ count: 3   │      │                                │
└────────────────────────────────┘      └────────────────────────────────┘
        5 messages                              2 messages
                                        (only latest per key!)
```

### Why Compaction is Powerful

1. **State Recovery**: On restart, read the entire compacted topic to rebuild your state. Since only the latest values exist, this is fast.

2. **Efficient Storage**: Old, superseded values are automatically cleaned up.

3. **Always Current**: Any new consumer reading from the beginning gets the current state, not historical values.

---

## New Topic: `watchlist`

The `watchlist` topic uses **log compaction**. This is perfect for our use case: tracking the *current* flag count per company, not the history of how it changed.

### Watchlist Record Schema

When you detect a flagged user, you'll produce a record like this:

```json
{
    "team": "team-1",
    "company": "Breitenberg and Sons",
    "flag_count": 3
}
```

The **key** of the message must be the company name (e.g., `"Breitenberg and Sons"`).

---

## TODOs

### 1. Track Companies in Memory

Before you can produce to the watchlist, you need to count how many flagged users belong to each company.

Add a dictionary at the top of your code to track flag counts:

```python
# Track how many times each company has been flagged
company_flags = {}
```

---

### 2. Update Your Processing Loop

When your filter matches (the same condition from Step 2), increment the company's count and check if it reaches the threshold:

```python
# Inside your message processing loop, after your filter matches:
company_name = parsed_message["company_name"]

# Increment the count (start at 0 if company not seen before)
company_flags[company_name] = company_flags.get(company_name, 0) + 1

if company_flags[company_name] >= 3:
    print(f"ALERT: {company_name} has reached the watchlist with {company_flags[company_name]} flags!")
```

---

### 3. Produce to Watchlist with KEY

Now produce to the `watchlist` topic. The key difference from Step 3 is that you **must include a key**:

```python
watchlist_record = {
    "team": TEAM_NAME,
    "company": company_name,
    "flag_count": company_flags[company_name]
}

# The key MUST be the company name (as bytes)
producer.send('watchlist',
              key=company_name.encode('utf-8'),
              value=bytes(json.dumps(watchlist_record), 'utf-8'))
```

**Important:** The key must be encoded as bytes using `.encode('utf-8')`.

Ask the instructor to check if they can see your watchlist messages!

---

## Compaction in Action

Now that you're producing to `watchlist`, observe what happens: each time you send an update for the same company, you're overwriting the previous value (from Kafka's compaction perspective). Even if you produce 100 updates for "Acme Corp", after compaction only the latest count remains—making state recovery fast and efficient.

---

## Check Your Work

| Leaderboard Shows | Meaning |
|-------------------|---------|
| 5️⃣ in Progress | Success! Watchlist message received |
| 1️⃣ 3️⃣ 4️⃣ 5️⃣ all visible | Tutorial complete! |

Ask the instructor to confirm your watchlist messages are appearing!

---

## Questions to Consider

1. What happens if two team members run the same code with the same team name?
2. Why must the message key match the company name in the value?
3. What would happen if your process crashes between incrementing the count and producing to Kafka?

---

## Bonus Challenge: State Recovery

What happens if your program crashes and restarts? Your `company_flags` dictionary would be empty, and you'd lose track of previous counts!

Since the `watchlist` topic is compacted, you can rebuild your state by reading from it on startup. The compacted topic only keeps the latest value per key, so you'll quickly get the current count for each company.

**Hint:** Create a second consumer that reads from `watchlist` with `auto_offset_reset='earliest'` before starting your main processing loop. Filter for your team's records and populate `company_flags` before processing new users.

This is an advanced challenge - ask your instructor for guidance if you get stuck!

---

## 🏆 Go for Champion!

Completed all 4 step achievements? Now unlock the bonus achievements to become the **first Champion**!

See [Step 0](/kafka-tutorial/docs/step-0.html#bonus-achievements) for the full list.

---

## Congratulations!

You've completed the Kafka tutorial!

Throughout these 5 steps, you've learned:
- **Step 1**: How to consume messages from Kafka
- **Step 2**: How to filter and transform data
- **Step 3**: How to produce messages to Kafka
- **Step 4**: How to scale with Consumer Groups
- **Step 5**: How to build stateful applications with keys and compaction

These are the building blocks for real-world stream processing systems. You're now ready to tackle more advanced Kafka concepts like exactly-once semantics, Kafka Streams, and schema registries.

Happy streaming!
