# Step 3: Load

With your detections in place, it’s time to notify another application by producing a Kafka message.

## TODOs

### 1. Create a Producer
To produce messages, you’ll need to create a [KafkaProducer](https://kafka-python.readthedocs.io/en/master/apidoc/KafkaProducer.html) at the top of your code. Reuse the same authentication details from the consumer.

**Required Parameters for the Producer:**
- `security_protocol`: `SASL_PLAINTEXT`
- `sasl_mechanism`: `PLAIN`
- `sasl_plain_username`: Your team name (e.g., `"team-1"`)
- `sasl_plain_password`: Ask your instructor
- `bootstrap_servers`: Ask your instructor
- `client_id`: Your team ID

---

### 2. Create a Message
Construct a Python dictionary with the following structure:

| **Key**    | **Value**                |
|------------|--------------------------|
| `customer` | The email of the customer |
| `type`     | The type from your team's row (see Step 2 table) |
| `reason`   | The reason from your team's row (see Step 2 table) |
| `team`     | Your team name           |

Once the dictionary is ready, convert it to JSON using [json.dumps](https://docs.python.org/3/library/json.html).

---

### 3. Push a Message
Use the Kafka topic `actions` to push your data. 

1. Use the `send` method from your producer to send the message. 
2. Before sending, convert the JSON string to bytes using:
   ```python
   bytes(your_json, "utf-8")
   ```

3. The send method will return a Future object.

---

## Understanding Asynchronous Sending

The `producer.send()` method is **non-blocking** - it returns immediately without waiting for the message to be delivered:

```python
future = producer.send('actions', value=bytes(json_message, 'utf-8'))
```

The message is queued in an internal buffer and sent asynchronously in the background. This is efficient but means you don't know immediately if the send succeeded.

### Confirming Delivery

To ensure your message was delivered, you can:

1. **Block and wait** for confirmation:
   ```python
   future = producer.send('actions', value=bytes(json_message, 'utf-8'))
   result = future.get(timeout=10)  # Blocks until sent or timeout
   print(f"Message sent to partition {result.partition} at offset {result.offset}")
   ```

2. **Flush before exiting** to ensure all buffered messages are sent:
   ```python
   producer.flush()  # Blocks until all messages are delivered
   ```

For this tutorial, calling `.get()` on each message helps you see immediate feedback. In production systems, you'd typically use callbacks or batch flushing for better performance.

For more details, refer to the [KafkaProducer guide](https://kafka-python.readthedocs.io/en/master/usage.html#kafkaproducer).

---

## Check Your Work

| Leaderboard Shows | Meaning |
|-------------------|---------|
| 3️⃣ in Progress | Success! Your filter + producer are working |
| ❌ in Errors | Invalid JSON - check `json.dumps()` |
| ❓ in Errors | Missing fields - need `customer`, `type`, `reason`, `team` |

Ask the instructor to confirm they can see your message!

## Next step

Congratulations, you learned how to produce a message in Kafka 🎉

You can now head to [step 4](/kafka-tutorial/docs/step-4.html)!
