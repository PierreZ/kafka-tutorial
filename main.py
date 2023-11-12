from kafka import KafkaConsumer
from kafka import KafkaProducer
from kafka.errors import KafkaError
import json

CONSUME_TOPIC="new_users"
PRODUCE_TOPIC="actions"

# You need to change these
BOOTSTRAP_SERVERS="localhost:9092"
TEAM_NAME = "team-1"
SASL_USERNAME="user"
SASL_PASSWORD= "password"

# Doc is over here: https://kafka-python.readthedocs.io/en/master/apidoc/KafkaConsumer.html#kafkaconsumer
consumer = KafkaConsumer(CONSUME_TOPIC, 
                         group_id=TEAM_NAME, 
                         client_id=TEAM_NAME,
                         security_protocol="SASL_SSL",
                         sasl_mechanism="PLAIN",
                         sasl_plain_username=SASL_USERNAME,
                         sasl_plain_password=SASL_PASSWORD,
                         bootstrap_servers=BOOTSTRAP_SERVERS)

for message in consumer:
    print ("%s:%d:%d: value=%s" % (message.topic, message.partition,
                                          message.offset, message.value))
