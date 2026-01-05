# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a 3-hour educational Apache Kafka tutorial designed for engineering students with no prior Kafka knowledge. It teaches message consumption and production through hands-on exercises, following a 5-step ETL pattern: Extract (consume) → Transform (filter) → Load (produce) → Scale (consumer groups).

## Tutorial Setup

- **Students**: Work on their own machines, writing code while following the tutorial at the GitHub Pages rendered version
- **Kafka cluster**: Deployed via Docker in the cloud, typically on port 443 to avoid firewall restrictions
- **Instructor**: Displays the TUI (terminal user interface) in class at all times to show real-time message flow
- **Local Docker setup** (`local-dev/`): Primarily for the instructor to run both the producer and TUI locally for testing purposes

## Architecture

Two main components:
- **Python consumer/producer** (`main.py`): Template code for tutorial participants using `kafka-python`
- **Rust producer** (`backend/producer/`): Generates fake user data and publishes to Kafka using `rdkafka`

Both connect via SASL_PLAINTEXT with PLAIN mechanism. Local development uses Docker (see `local-dev/`).

## Build and Run Commands

### Rust Producer

```bash
# Build
cargo build --release --manifest-path backend/producer/Cargo.toml

# Run (requires config.toml with broker credentials)
./backend/producer/target/release/producer -c backend/producer/config.toml

# Run tests
cargo test --manifest-path backend/producer/Cargo.toml
```

### Python Tutorial

```bash
# Install dependencies
pip install kafka-python

# Run consumer
python main.py
```

### Kafka Server (Docker)

```bash
# Start container (local)
./start-kafka.sh localhost

# Start container (cloud - replace with your VM's public IP)
./start-kafka.sh 1.2.3.4

# Create topics and ACLs
./setup-kafka.sh
```

## Configuration

Producer config (`backend/producer/config.toml`):
- `interval_millis`: Message send interval
- `topic`: Target Kafka topic (default: "new_users")
- `brokers`: Kafka bootstrap servers
- `username`/`password`: SASL credentials

## Key Data Structure

The Rust producer generates `User` objects with fields: email, credit_card_number, company_name, company_slogan, industry, user_name, avatar, name, profession, field, premium, credit, time_zone, user_agent, pack.
