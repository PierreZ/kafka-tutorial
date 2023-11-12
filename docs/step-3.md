# Step 3: Load

Now that you have detected something, let's notify another app.

## TODOS

### Create a producer

You need to create a [KafkaProducer](https://kafka-python.readthedocs.io/en/master/apidoc/KafkaProducer.html) at the top of your code.

To produce, you will need to set the parameters for the Producer:

* security_protocol=SASL_SSL
* sasl_mechanism=PLAIN
* sasl_plain_username="user"
* sasl_plain_password="toto"
* bootstrap_servers="localhost:8080"
* client_id=`your team ID`

You can reuse the same auth from the consumer.

### Create a message

You need to create a Python dictionary. The dictionary must have these parameters:

| key      | value                    |
|----------|--------------------------|
| customer | email of the customer    |
| type     | the type of your team    |
| reason   | the reason of your team  |
| team     | your team                |

Once you have created the dictionary, the JSON can be created with [json.dumps](https://docs.python.org/3/library/json.html).

### Push a message

The topic `actions` is the one where you should be pushing data.

You can use the `send` method from your producer. You will need to convert your data from string to bytes by using `bytes(yourJson, "utf-8")`.

The producer will create a Future. You can find some tips on how to manipulate the Future [here](https://kafka-python.readthedocs.io/en/master/usage.html#kafkaproducer).

Ask the instructor to check if he can see your message!

## Next step

Congratulations, you learned how to produce a message in Kafka 🎉

You can now head to [step 4](/kafka-tutorial/docs/step-4.html)!