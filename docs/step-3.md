# Push an action message

You now have a message to push.

### `actions`

The topic `action` is the one where you should be pushing data. It will accept data in the JSON format:

```json
{
  "type": "",
  "team": "",
  "reason": ""
}
```

Each team will have to push the corresponding json to the same topic. You need to use the type and the reason from your team.

To produce, you will need to set the parameters for the Producer:

* bootstrap.servers={{ CLUSTER_ENDPOINT }}
* security.protocol=SASL_SSL
* sasl.mechanisms=PLAIN
* sasl.username={{ CLUSTER_API_KEY }}
* sasl.password={{ CLUSTER_API_SECRET }}
