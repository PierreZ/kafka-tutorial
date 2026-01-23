# Step 4: Scale-up

In this step, we'll explore how to scale your application by running multiple instances of your program. Kafka's **Consumer Groups** enable load balancing and fault tolerance by distributing the processing of messages across multiple consumers within the same group.

## Understanding Consumer Groups

### What is a Consumer Group?

A **Consumer Group** is a set of consumers that work together to consume messages from a topic. The key insight is that **each partition is assigned to exactly one consumer within a group**—no two consumers in the same group ever read from the same partition.

Think of it like a team splitting up work: if you have 3 partitions and 2 team members, one person handles 2 partitions while the other handles 1.

### The group.id

Your consumer's `group_id` (e.g., `"team-1"`) identifies which Consumer Group it belongs to. All consumers with the same `group_id` are part of the same group and share the workload.

```
                    Topic: new_users (5 partitions)
            ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐
            │  P0   │ │  P1   │ │  P2   │ │  P3   │ │  P4   │
            └───┬───┘ └───┬───┘ └───┬───┘ └───┬───┘ └───┬───┘
                │        │        │        │        │
    ┌───────────┼────────┼────────┼────────┼────────┼─────────────┐
    │           ▼        ▼        ▼        ▼        ▼             │
    │  Consumer Group: team-1                                     │
    │                                                             │
    │  With 1 consumer:                                           │
    │  ┌───────────────────────────────────────────┐             │
    │  │         Consumer A (your laptop)          │             │
    │  │          reads P0, P1, P2, P3, P4         │             │
    │  └───────────────────────────────────────────┘             │
    │                                                             │
    │  With 2 consumers (after colleague joins):                  │
    │  ┌─────────────────────┐  ┌─────────────────────┐          │
    │  │     Consumer A      │  │     Consumer B      │          │
    │  │   reads P0, P1, P2  │  │    reads P3, P4     │          │
    │  └─────────────────────┘  └─────────────────────┘          │
    └─────────────────────────────────────────────────────────────┘
```

### What is Rebalancing?

**Rebalancing** is the process where Kafka redistributes partitions among consumers. It happens when:
- A new consumer joins the group
- A consumer leaves the group (crashes, shuts down, or times out)
- New partitions are added to the topic

During a rebalance:
1. All consumers in the group temporarily stop reading
2. Kafka reassigns partitions based on the number of active consumers
3. Each consumer receives its new partition assignment
4. Consumers resume reading from where they left off

This typically takes a few seconds—you might notice a brief pause in message processing.

### Offset Tracking: Remembering Where You Left Off

Kafka tracks the **committed offset** for each partition within each Consumer Group. This is how Kafka knows where each group left off, even if consumers restart.

```
Consumer Group: team-1
┌─────────────┬──────────────────┐
│  Partition  │ Committed Offset │
├─────────────┼──────────────────┤
│     P0      │       142        │  ← Last processed message
│     P1      │       98         │
│     P2      │       201        │
│     P3      │       67         │
│     P4      │       189        │
└─────────────┴──────────────────┘
```

When a consumer (re)starts, it asks: "What was the last offset I committed for each partition?" and resumes from there.

### Where Are Offsets Stored?

Kafka stores committed offsets in a special internal topic called `__consumer_offsets`. This is a compacted topic (see Step 5) that keeps the latest offset for each `(group_id, topic, partition)` combination.

**How offset commits work:**

1. Your consumer processes messages (e.g., offsets 0-9)
2. By default, `kafka-python` **auto-commits** every 5 seconds
3. The committed offset is written to `__consumer_offsets`
4. If your consumer crashes and restarts, it reads from `__consumer_offsets` to find where it left off

**What this means for your application:**

- **Messages may be reprocessed**: If your consumer crashes between processing a message and the next auto-commit, those messages will be redelivered
- **At-least-once delivery**: Kafka guarantees you'll see every message at least once, but possibly more than once
- **Design for idempotency**: Your processing logic should handle duplicate messages gracefully

> **Note:** You can disable auto-commit and manually commit offsets for finer control, but that's beyond this tutorial's scope.

### Key Insight: Max Parallelism = Partition Count

You **cannot have more active consumers than partitions**. If you have 3 partitions and 4 consumers, one consumer will sit idle with nothing to read.

| Consumers | Partition Assignment | Notes |
|-----------|---------------------|-------|
| 1 | A: P0, P1, P2, P3, P4 | One consumer handles everything |
| 2 | A: P0, P1, P2  B: P3, P4 | Work is split (3+2) |
| 3 | A: P0, P1  B: P2, P3  C: P4 | Better distribution (2+2+1) |
| 5 | A: P0  B: P1  C: P2  D: P3  E: P4 | Maximum parallelism |
| 6 | A: P0  B: P1  C: P2  D: P3  E: P4  F: (idle) | Extra consumer is wasted! |

This is why partition count is an important decision when creating topics.

---

## TODOs

### 1. Simulate Increased Load
Ask the instructor to increase the message production rate to simulate higher traffic.

### 2. Collaborate with a Colleague
- Share your program code with a colleague.
- Both of you should configure the consumer to use the **same Consumer Group ID**.

### 3. Run Multiple Instances
- Run the program on multiple laptops or machines, ensuring all instances belong to the same Consumer Group.
- Observe how Kafka distributes the workload among the consumers.

### 4. Test Fault Tolerance
- Stop one of the running instances (simulate an instance failure).
- Observe how Kafka reassigns the partitions previously handled by the stopped instance to the remaining active consumers.

---

## Questions to Consider
1. **What happens when a second program joins the Consumer Group?**
   - How are partitions redistributed among the consumers?
   - Do both consumers receive all messages or only a subset?

2. **What happens when a program leaves the Consumer Group?**
   - How does Kafka handle the rebalancing of partitions?
   - Does the remaining consumer pick up the workload from the instance that left?

---

## Key Takeaways
- Consumer Groups allow you to scale your application horizontally by adding more consumers.
- Kafka automatically balances partitions among consumers within a group, making it resilient to failures and capable of handling increased load.

---

## Check Your Work

| Leaderboard Shows | Meaning |
|-------------------|---------|
| ⚖️ in Progress | Success! 2+ consumers active |
| Only 🔌 📤 | Only one consumer running |

---

Congratulations, you've learned how to distribute and scale your Kafka-based program using Consumer Groups!

You can now head to [step 5](/kafka-tutorial/docs/step-5.html) to learn about stateful processing!
