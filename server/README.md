# Kafka Tutorial Server

NixOS-based OpenStack image that bootstraps Apache Kafka in KRaft mode for the tutorial.

## Features

- Apache Kafka in KRaft mode (no ZooKeeper)
- SASL/PLAIN authentication on port 443
- Pre-configured topics: `new_users` and `actions`
- 15 team credentials ready to use
- OpenStack cloud-init support

## Building the Image

### Prerequisites

- Nix with flakes enabled
- x86_64-linux system (or cross-compilation setup)

### Build Commands

```bash
cd server

# Build OpenStack QCOW2 image
nix build .#openstack

# Result will be in ./result/
ls -la result/
```

For local testing with QEMU:

```bash
nix build .#qcow
```

## Deploying to OpenStack

```bash
# Upload the image
openstack image create \
  --disk-format qcow2 \
  --container-format bare \
  --file result/nixos.qcow2 \
  --property hw_disk_bus=virtio \
  --property hw_vif_model=virtio \
  kafka-tutorial

# Create an instance
openstack server create \
  --image kafka-tutorial \
  --flavor m1.medium \
  --network <your-network> \
  --key-name <your-keypair> \
  kafka-server
```

## Connection Details

### Kafka Broker

- **Host**: `<server-ip>`
- **Port**: `443`
- **Protocol**: `SASL_PLAINTEXT`
- **SASL Mechanism**: `PLAIN`

### Topics

| Topic | Purpose | Partitions |
|-------|---------|------------|
| `new_users` | Source topic with user data | 3 |
| `actions` | Destination topic for processed data | 3 |

## Team Credentials

| Team | Username | Password |
|------|----------|----------|
| 1 | team-1 | team-1-secret |
| 2 | team-2 | team-2-secret |
| 3 | team-3 | team-3-secret |
| 4 | team-4 | team-4-secret |
| 5 | team-5 | team-5-secret |
| 6 | team-6 | team-6-secret |
| 7 | team-7 | team-7-secret |
| 8 | team-8 | team-8-secret |
| 9 | team-9 | team-9-secret |
| 10 | team-10 | team-10-secret |
| 11 | team-11 | team-11-secret |
| 12 | team-12 | team-12-secret |
| 13 | team-13 | team-13-secret |
| 14 | team-14 | team-14-secret |
| 15 | team-15 | team-15-secret |
| Admin | admin | admin-secret |

## Python Client Example

```python
from kafka import KafkaConsumer, KafkaProducer

# Replace with your server IP and team credentials
BOOTSTRAP_SERVERS = "<server-ip>:443"
TEAM_NAME = "team-1"
SASL_USERNAME = "team-1"
SASL_PASSWORD = "team-1-secret"

# Consumer
consumer = KafkaConsumer(
    'new_users',
    bootstrap_servers=BOOTSTRAP_SERVERS,
    group_id=TEAM_NAME,
    security_protocol='SASL_PLAINTEXT',
    sasl_mechanism='PLAIN',
    sasl_plain_username=SASL_USERNAME,
    sasl_plain_password=SASL_PASSWORD,
    auto_offset_reset='earliest',
    value_deserializer=lambda m: json.loads(m.decode('utf-8'))
)

# Producer
producer = KafkaProducer(
    bootstrap_servers=BOOTSTRAP_SERVERS,
    security_protocol='SASL_PLAINTEXT',
    sasl_mechanism='PLAIN',
    sasl_plain_username=SASL_USERNAME,
    sasl_plain_password=SASL_PASSWORD,
    value_serializer=lambda v: json.dumps(v).encode('utf-8')
)
```

## SSH Access

The image uses cloud-init to inject SSH keys. Connect with:

```bash
ssh nixos@<server-ip>
```

## Troubleshooting

### Check Kafka service status

```bash
ssh nixos@<server-ip>
sudo systemctl status apache-kafka
sudo journalctl -u apache-kafka -f
```

### Check topic creation

```bash
ssh nixos@<server-ip>
sudo systemctl status kafka-create-topics
sudo journalctl -u kafka-create-topics
```

### List topics manually

```bash
kafka-topics.sh --list \
  --bootstrap-server localhost:443 \
  --command-config /etc/kafka/admin.properties
```

### Test producing a message

```bash
echo '{"test": "message"}' | kafka-console-producer.sh \
  --bootstrap-server localhost:443 \
  --topic new_users \
  --producer.config /etc/kafka/admin.properties
```
