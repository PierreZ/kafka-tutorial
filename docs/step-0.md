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
                                  │ team_stats (Step 5)
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

## Environment Setup

Choose one of the following options to set up your development environment:

### Option A: Virtualenv (for experienced users)

```bash
git clone https://github.com/PierreZ/kafka-tutorial.git
cd kafka-tutorial
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -r requirements.txt
```

### Option B: Dev Container (recommended)

1. Install [Docker](https://www.docker.com/products/docker-desktop) and [VS Code](https://code.visualstudio.com/)
2. Install the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
3. Clone the repository and open it in VS Code
4. Click "Reopen in Container" when prompted (or use the command palette: `Dev Containers: Reopen in Container`)
5. Wait for the container to build - your environment is ready!

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
| 1 | **Connected** | 🔌 | Consumer group becomes active |
| 3 | **First Load** | 📤 | Produce first valid action message |
| 4 | **Scaled** | ⚖️ | Have 2+ consumers in your group |
| 5 | **Stats Published** | 📊 | Produce first stats message with key |

> Step 2 (Transform) has no achievement - your filter is verified when 📤 unlocks.

### Reading the Leaderboard

- **Progress column**: Shows 🔌 📤 ⚖️ 📊 for steps (⚪ for incomplete)
- **Team color**: Green (all 4 steps) → Yellow (3) → Cyan (2) → Blue (1) → Gray (none)

---

Now that you have the context, you're ready to dive into the next step! Continue on to [Step 1](/kafka-tutorial/docs/step-1.html) to get started.
