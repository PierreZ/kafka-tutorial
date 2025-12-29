# Step 5: Stateful Processing (Watchlist)

Your fraud detection system from the previous steps is working well! But your mentor has a new observation: sometimes multiple flagged users come from the **same company**. When a company has 3 or more flagged users, it's worth investigating further.

Your task is to build a **watchlist** that tracks companies with suspicious activity.

---

## What You'll Learn

In this step, you'll learn about:
- **Message Keys**: How to produce messages with a key, not just a value
- **Log Compaction**: A special topic configuration that keeps only the latest value per key
- **Stateful Processing**: Maintaining counts in memory across messages

---

## New Topic: `watchlist`

The `watchlist` topic is special - it uses **log compaction**. Unlike regular topics that keep all messages, a compacted topic only retains the **latest message for each key**.

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

## Why Compaction Works

Consider what happens when you flag the same company multiple times:

```
Key: "Acme Corp" → Value: {"flag_count": 1}
Key: "Acme Corp" → Value: {"flag_count": 2}
Key: "Acme Corp" → Value: {"flag_count": 3}
```

After compaction runs, only the **latest** record for each key is kept:

```
Key: "Acme Corp" → Value: {"flag_count": 3}
```

This means on restart, you only need to read a small number of messages to fully restore your state!

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
