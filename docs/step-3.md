# Step 3: Load

With your detections in place, it’s time to notify another application by producing a Kafka message.

## TODOs

### 1. Create a Producer
To produce messages, you’ll need to create a [KafkaProducer](https://kafka-python.readthedocs.io/en/master/apidoc/KafkaProducer.html) at the top of your code. Reuse the same authentication details from the consumer.

**Required Parameters for the Producer:**
- `security_protocol`: `SASL_PLAINTEXT`
- `sasl_mechanism`: `PLAIN`
- `sasl_plain_username`: `"user"`
- `sasl_plain_password`: `"toto"`
- `bootstrap_servers`: `"localhost:8080"`
- `client_id`: Your team ID

---

### 2. Create a Message
Construct a Python dictionary with the following structure:

| **Key**    | **Value**                |
|------------|--------------------------|
| `customer` | The email of the customer |
| `type`     | The type assigned to your team |
| `reason`   | The reason assigned to your team |
| `team`     | Your team name           |

Once the dictionary is ready, convert it to JSON using [json.dumps](https://docs.python.org/3/library/json.html).

---

### 3. Push a Message
Use the Kafka topic `actions` to push your data. 

1. Use the `send` method from your producer to send the message. 
2. Before sending, convert the JSON string to bytes using:
   ```python
   bytes(your_json, "utf-8")

3. The send method will return a Future object. For help with handling Futures, refer to this [guide](https://kafka-python.readthedocs.io/en/master/usage.html#kafkaproducer).

Ask the instructor to check if he can see your message!

## Next step

Congratulations, you learned how to produce a message in Kafka 🎉

You can now head to [step 4](/kafka-tutorial/docs/step-4.html)!
