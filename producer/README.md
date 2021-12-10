# Producer

This producer is sending fake messages to a Kafka topic.

## How-to

```shell
cargo build --release
./target/release/producer -c $PWD/config.toml
``