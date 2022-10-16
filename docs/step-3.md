# Push an action message

Now that you have a user, we now need to push a message. There is an microservice waiting for actions to do.

## TODOs

* Create a Producer with the right parameters (see below)
* serialize the action as a JSON
* produce a message in the `actions` topic
* print the resulting offsets/partitions

## Topics 

### `actions`

The topic `actions` is the one where you should be pushing data. It will accept data in the JSON format:

```json
{
  "type": "",
  "team": "",
  "reason": "",
  "customer": "customer's mail"
}
```

## Producer configuration

Each team will have to push the corresponding json to the same topic. You need to use the type and the reason from your team.

To produce, you will need to set the parameters for the Producer:

* security.protocol=SASL_SSL
* sasl.mechanisms=PLAIN
* bootstrap.servers=`the cluster endpoint provided by the instructor`
* sasl.username=`the api key provided by the instructor`
* sasl.password=`the api secret provided by the instructor`
* client.id=`your team ID`
  
⚠️⚠️⚠️ Don't forget to set a client.id for production!

## Questions before moving on

* how many partitions does the topic `actions` have?

## Next step

Congratulations, you learned how to produce a message in Kafka 🎉

You can now head to [step 4](/kafka-tutorial/docs/step-4.html)!