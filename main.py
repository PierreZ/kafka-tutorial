from kafka import KafkaConsumer
from kafka import KafkaProducer
from kafka.errors import KafkaError
import json

CONSUME_TOPIC="new_users"
PRODUCE_TOPIC="actions"

# You need to change this
TEAM_NAME = "team-1"

# Do not change this
SASL_USERNAME = TEAM_NAME
SASL_PASSWORD = f"{TEAM_NAME}-secret"
BOOTSTRAP_SERVERS="152.228.210.193:443"

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
