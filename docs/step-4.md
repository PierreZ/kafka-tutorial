# Step 4: Scale-up

In this step, we'll explore how to scale your application by running multiple instances of your program. Kafka's **Consumer Groups** enable load balancing and fault tolerance by distributing the processing of messages across multiple consumers within the same group.

## Understanding Consumer Groups

### What is a Consumer Group?

A **Consumer Group** is a set of consumers that work together to consume messages from a topic. The key insight is that **each partition is assigned to exactly one consumer within a group**—no two consumers in the same group ever read from the same partition.

Think of it like a team splitting up work: if you have 3 partitions and 2 team members, one person handles 2 partitions while the other handles 1.

### The group.id

Your consumer's `group_id` (e.g., `"team-1"`) identifies which Consumer Group it belongs to. All consumers with the same `group_id` are part of the same group and share the workload.

```
                    Topic: new_users (3 partitions)
                    ┌───────┐ ┌───────┐ ┌───────┐
                    │  P0   │ │  P1   │ │  P2   │
                    └───┬───┘ └───┬───┘ └───┬───┘
                        │        │        │
    ┌───────────────────┼────────┼────────┼───────────────────┐
    │                   ▼        ▼        ▼                   │
    │  Consumer Group: team-1                                 │
    │                                                         │
    │  With 1 consumer:                                       │
    │  ┌─────────────────────────────────────┐               │
    │  │     Consumer A (your laptop)        │               │
    │  │        reads P0, P1, P2             │               │
    │  └─────────────────────────────────────┘               │
    │                                                         │
    │  With 2 consumers (after colleague joins):              │
    │  ┌─────────────────┐  ┌─────────────────┐              │
    │  │   Consumer A    │  │   Consumer B    │              │
    │  │   reads P0, P1  │  │    reads P2     │              │
    │  └─────────────────┘  └─────────────────┘              │
    └─────────────────────────────────────────────────────────┘
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
└─────────────┴──────────────────┘
```

When a consumer (re)starts, it asks: "What was the last offset I committed for each partition?" and resumes from there.

### Key Insight: Max Parallelism = Partition Count

You **cannot have more active consumers than partitions**. If you have 3 partitions and 4 consumers, one consumer will sit idle with nothing to read.

| Consumers | Partition Assignment | Notes |
|-----------|---------------------|-------|
| 1 | A: P0, P1, P2 | One consumer handles everything |
| 2 | A: P0, P1  B: P2 | Work is split |
| 3 | A: P0  B: P1  C: P2 | Maximum parallelism |
| 4 | A: P0  B: P1  C: P2  D: (idle) | Extra consumer is wasted! |

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
| 4️⃣ in Progress | Success! 2+ consumers active |
| Only 1️⃣ 3️⃣ | Only one consumer running |

---

## Bonus Challenges

### 🔬 Partition Explorer

Try adding a **third consumer** to your group. What happens?

With only 3 partitions in the `new_users` topic, one consumer will sit idle—it has no partition to read from! This teaches a key Kafka lesson: **partitions limit parallelism**. You can't have more active consumers than partitions.

> **Unlock condition**: Have 3+ consumers in your group

### 🚀 Lag Buster

**Consumer lag** is the difference between the latest message produced and the last message your consumer read. High lag means you're falling behind!

Try this experiment:
1. Stop all your consumers for 30+ seconds
2. Watch the leaderboard—your lag counter will climb
3. Restart your consumer and watch it catch up
4. When lag hits 0, you unlock **Lag Buster**!

> **Unlock condition**: Build up 100+ lag, then consume back to 0

---

Congratulations, you've learned how to distribute and scale your Kafka-based program using Consumer Groups! 🎉

You can now head to [step 5](/kafka-tutorial/docs/step-5.html) to learn about stateful processing!
