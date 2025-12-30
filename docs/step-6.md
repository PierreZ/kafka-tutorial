# Step 6: Achievements & Leaderboard

Congratulations on completing the Kafka tutorial! Your instructor has a real-time leaderboard that tracks your team's progress and awards achievements based on your actions.

## How Scoring Works

The leaderboard validates every action your team produces to the `actions` topic by checking:

1. **Valid JSON format** - Your message must be valid JSON
2. **Required fields** - Must include `customer`, `type`, `reason`, and `team`
3. **Correct team name** - Must match `team-N` format (e.g., `team-1`)
4. **User exists** - The `customer` must be a user from the `new_users` topic
5. **Correct filter match** - The user must actually match your team's filter criteria
6. **Correct type/reason** - Must match your team's assigned values
7. **No duplicates** - Can't flag the same customer twice

Each valid action earns **10 points**.

## Achievements

### Progress Achievements

Earn points by processing messages correctly!

| Badge | Name | How to Unlock | Points |
|-------|------|---------------|--------|
| 🐣 | **First Steps** | Produce your first valid action | 10 |
| 🔥 | **Fifty** | Produce 50 valid actions | 100 |
| 💯 | **Century** | Produce 100 valid actions | 200 |
| ⚡ | **Streak 10** | 10 consecutive correct actions (no mistakes!) | 50 |

### Mistake Achievements

These badges are educational - they help you identify what went wrong. They award **0 points** but are visible on the leaderboard.

| Badge | Name | What Went Wrong |
|-------|------|-----------------|
| ❌ | **Parse Error** | Your message wasn't valid JSON |
| 👻 | **Ghost User** | You flagged a `customer` that doesn't exist in `new_users` |
| 2️⃣ | **Duplicate** | You already flagged this customer |
| ❓ | **Missing Fields** | Your action was missing `customer`, `type`, `reason`, or `team` |
| 🙈 | **False Positive** | The user doesn't match your team's filter criteria |

### Infrastructure Achievements

Earn bonus points by using Kafka's consumer group features!

| Badge | Name | How to Unlock | Points |
|-------|------|---------------|--------|
| 🔌 | **Connected** | Your consumer group is active and consuming | 25 |
| 👥 | **Scaled** | Run 2 or more consumers in your consumer group | 50 |

## Tips for Maximizing Your Score

1. **Test locally first** - Make sure your filter logic works before deploying
2. **Check your JSON** - Use `json.dumps()` in Python or equivalent
3. **Verify field names** - Double-check `customer`, `type`, `reason`, `team`
4. **Use exact values** - The `type` and `reason` must match exactly (case-sensitive!)
5. **Scale up** - Run multiple consumers in your group for the Scaled achievement
6. **Keep your streak** - Consecutive correct actions build toward Streak 10

## Leaderboard Display

The instructor's dashboard shows:

```
+--------------------------------------------------+
|           KAFKA TUTORIAL LEADERBOARD             |
+--------------------------------------------------+
| TEAM RANKINGS              | CONSUMER GROUPS     |
| #  Team   Score C/I  Strk  | team-1: Active(2)   |
| 1. team-3 1250 125/5  15   | team-2: Empty       |
| 2. team-1  980  98/12  8   | team-3: Active(1)   |
+----------------------------+---------------------+
| RECENT ACHIEVEMENTS                              |
| [14:32] team-3: ⚡ Streak 10!                    |
| [14:31] team-1: 🐣 First Steps!                  |
+--------------------------------------------------+
```

- **Score** - Total points earned
- **C/I** - Correct/Incorrect action counts
- **Strk** - Current streak of consecutive correct actions
- **Consumer Groups** - Shows if your team's consumers are connected

## State Persistence

The leaderboard persists state to a Kafka topic (`scorer_state`), demonstrating the same pattern you learned in Step 5 with the watchlist. If the leaderboard restarts, your scores are preserved!

---

Good luck, and may the best team win! 🏆
