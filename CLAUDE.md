# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an educational Apache Kafka tutorial that teaches message consumption and production through hands-on exercises. The tutorial follows a 5-step ETL pattern: Extract (consume) → Transform (filter) → Load (produce) → Scale (consumer groups).

## Architecture

Two main components:
- **Python consumer/producer** (`main.py`): Template code for tutorial participants using `kafka-python`
- **Rust producer** (`producer/`): Generates fake user data and publishes to Kafka using `rdkafka`

Both connect via SASL_SSL with PLAIN mechanism to Confluent Cloud.

## Build and Run Commands

### Rust Producer

```bash
# Build
cargo build --release --manifest-path producer/Cargo.toml

# Run (requires config.toml with broker credentials)
./producer/target/release/producer -c producer/config.toml

# Run tests
cargo test --manifest-path producer/Cargo.toml
```

### Python Tutorial

```bash
# Install dependencies
pip install kafka-python

# Run consumer
python main.py
```

### Nix Development Environment

```bash
cd producer
direnv allow  # or: nix develop
```

## Configuration

Producer config (`producer/config.toml`):
- `interval_millis`: Message send interval
- `topic`: Target Kafka topic (default: "new_users")
- `brokers`: Kafka bootstrap servers
- `username`/`password`: SASL credentials

## Key Data Structure

The Rust producer generates `User` objects with fields: email, credit_card_number, company_name, company_slogan, industry, user_name, avatar, name, profession, field, premium, credit, time_zone, user_agent, pack.
