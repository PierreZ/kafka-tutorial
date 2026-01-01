# Step 0: Context and Helpers

## Situation

You’ve recently joined a fast-growing startup as an intern. Your mentor has tasked you with developing small applications to help the company manage its operations. The startup is seeing a rapid increase in new user registrations, and there’s a need to process these registrations efficiently.

Each new registration is sent to Kafka, and your task is to handle various small operations triggered by these messages.

---

## Architecture

Each team will develop an application that follows a common pattern known as **ETL** (Extract, Transform, Load):

1. **Extract** data from Kafka messages.
2. **Transform** the data by applying necessary processing or validation.
3. **Load** the results back into Kafka by producing new messages.

```
┌──────────┐   new_users   ┌─────────────┐   actions    ┌─────────────┐
│ Producer │ ────────────► │  Your App   │ ───────────► │ Leaderboard │
└──────────┘               │  (Python)   │              └─────────────┘
                           └─────────────┘
                                  │
                                  │ watchlist (Step 5)
                                  ▼
                           ┌─────────────┐
                           │  Compacted  │
                           │    Topic    │
                           └─────────────┘
```

These applications can be written in any language, but for this tutorial, support will be provided for the following languages:
- Java
- Go
- Python
- Rust

You can use the online Python environment, accessible through the link below, to get started with the tutorial:

[![Open in GitPod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/PierreZ/kafka-tutorial)

---

## Libraries

To connect to Kafka from your application, you can use the following libraries depending on the language you choose:

- [Python client](https://kafka-python.readthedocs.io/en/master/)
- [Go client](https://github.com/Shopify/sarama)
- [Rust client](https://github.com/fede1024/rust-rdkafka)
- [Node client](https://www.npmjs.com/package/kafka-node)
- [Java client](https://search.maven.org/#artifactdetails%7Corg.apache.kafka%7Ckafka-clients%7C1.1.0%7Cjar)

---

## What Data Will You See?

The producer generates fake user data with some patterns you should know about:

| Field | Distribution | Relevant For |
|-------|-------------|--------------|
| `avatar` | ~90% robohash URLs, ~10% `example.org` | Team-13 (invalid avatar) |
| `name` | ~90% generated names, ~10% "John Doe" | Team-14 (suspicious name) |
| `premium` | 50% true, 50% false | Team-7 (VIP users) |
| `pack` | ~90% "small", ~10% "free" | Team-15 (upgrade free) |
| `credit` | Range: -20 to +20 | Team-7 (>10), Team-8 (<-15) |

This helps you understand expected match rates - if your filter never matches, double-check your logic!

---

## Leaderboard & Achievements

Your instructor displays a real-time leaderboard that tracks your team's progress!

### Step Achievements

| Step | Achievement | Emoji | How to Unlock |
|------|-------------|-------|---------------|
| 1 | **Connected** | 1️⃣ | Consumer group becomes active |
| 3 | **First Load** | 3️⃣ | Produce first valid action message |
| 4 | **Scaled** | 4️⃣ | Have 2+ consumers in your group |
| 5 | **Watchlist** | 5️⃣ | Produce first watchlist message |

> Step 2 (Transform) has no achievement - your filter is verified when 3️⃣ unlocks.

### Error Indicators

| Emoji | Error | What Went Wrong |
|-------|-------|-----------------|
| ❌ | **Parse Error** | Invalid JSON format |
| ❓ | **Missing Fields** | Missing `customer`, `type`, `reason`, or `team` |

### Reading the Leaderboard

- **Progress column**: Shows 1️⃣ 3️⃣ 4️⃣ 5️⃣ for completed steps (⬜ for incomplete)
- **Errors column**: Shows error counts (e.g., ❌x2)
- **Team color**: Green (all 4) → Yellow (3) → Cyan (2) → Blue (1) → Gray (none)

---

Now that you have the context, you're ready to dive into the next step! Continue on to [Step 1](/kafka-tutorial/docs/step-1.html) to get started.
