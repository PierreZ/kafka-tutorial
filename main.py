from kafka import KafkaConsumer
from kafka import KafkaProducer
from kafka.errors import KafkaError
import json

CONSUME_TOPIC="new_users"
PRODUCE_TOPIC="actions"

# You need to change this
BOOTSTRAP_SERVERS="152.228.210.193:443"
TEAM_NAME = "team-1"
SASL_USERNAME = TEAM_NAME
SASL_PASSWORD = f"{TEAM_NAME}-secret"

# Doc is over here: https://kafka-python.readthedocs.io/en/master/apidoc/KafkaConsumer.html#kafkaconsumer
consumer = KafkaConsumer(CONSUME_TOPIC, 
                         group_id=TEAM_NAME, 
                         client_id=TEAM_NAME,
                         security_protocol="SASL_PLAINTEXT",
                         sasl_mechanism="SCRAM-SHA-256",
                         sasl_plain_username=SASL_USERNAME,
                         sasl_plain_password=SASL_PASSWORD,
                         bootstrap_servers=BOOTSTRAP_SERVERS)

for message in consumer:
    print ("%s:%d:%d: value=%s" % (message.topic, message.partition,
                                          message.offset, message.value))

    # === Step 1: Parse the JSON ===
    # TODO: Decode and parse the message
    # parsed = json.loads(message.value.decode('utf-8'))

    # === Step 2: Apply your team's filter ===
    # TODO: Check if the user matches your team's criteria
    # if matches_filter(parsed):
    #     ...

    # === Step 3: Produce to actions topic ===
    # TODO: Create a producer and send matching users
    # See step-3 docs for KafkaProducer configuration
