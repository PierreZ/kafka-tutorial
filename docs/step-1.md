# Step 1: Extract

The very first step to do is to connect to Kafka and display all incoming messages. Let's do this!

## Topics to read

### `new_users`

The main topic is named "new_users". Here's an example of the JSON pushed:

```json
{
    "email": "hugh_atque@hotmail.com",
    "credit_card_number": "373947589498776",
    "company_name": "Breitenberg and Sons",
    "company_slogan": "Open-architected directional adapter",
    "industry": "Market Research",
    "user_name": "ilene_quaerat",
    "avatar": "https://robohash.org/marcus_omnis.png?size=50x50",
    "name": "Roslyn Dicki",
    "profession": "advocate",
    "field": "Mining",
    "premium": true,
    "credit": -7,
    "time_zone": "Pacific/Pago_Pago",
    "user_agent": "Mozilla/5.0 (Windows; U; MSIE 9.0; WIndows NT 9.0; en-US))",
    "pack": "small"
}
```

## TODOs

### Read Kafka

By loading the GitPod, you should have the boilerplate already setup.

You should be able to run the code by replacing `BOOTSTRAP_SERVERS`, `TEAM_NAME`, `SASL_USERNAME` and `SASL_PASSWORD`. When running the code, you should see the messages.

### Questions

* how many partitions does the topic `new_users` have?

## Parsing the JSON

Now that we are displaying the whole message, we should interpret the `value` of the message. It is a JSON, so in Python, you can use the [json package and the json.loads function](https://docs.python.org/3/library/json.html).

To prove that you managed to parse the JSON, try printing only the email.

## Next step

Congratulations, you learned how to consume a message in Kafka 🎉
You can now continue on [step-2](/kafka-tutorial/docs/step-2.html)!
