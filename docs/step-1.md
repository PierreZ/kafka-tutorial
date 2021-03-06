# Step 1: Consume Kafka

You first need to consume Kafka. so the instructor should have displayed:

* cluster endpoint,
* api key,
* api secret.

## TODOs 

* Create Consumer with the right parameters(see below)
* Loop through all messages
* print the different element of a Kafka message:
  * value,
  * partition,
  * offsets,
  * topic,
* deserialize the Kafka payload in a structure/JSON

## Consumer's configuration 

We will be using a topic from Confluent Cloud, so you will need to pass those parameters to the Consumer:

* bootstrap.servers=$CLUSTER_ENDPOINT
* security.protocol=SASL_SSL
* sasl.mechanisms=PLAIN
* sasl.username=$CLUSTER_API_KEY
* sasl.password=$CLUSTER_API_SECRET
* group.id=$TEAM_ID
* client.id=$TEAM_ID

⚠️⚠️⚠️ Don't forget to set both a group.id and a client.id for consumption!

## Topics

### `new_user`
The main topic is named "new_user". Here's an example of the JSON pushed:

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


## Questions before moving on

* how many partitions does the topic `new_users` have?

## Next step

Congratulations, you learned how to consume a message in Kafka 🎉
You can now continue on [step-2](/kafka-tutorial/docs/step-2.html)!