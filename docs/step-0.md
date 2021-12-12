# Step 0: context and helpers

## Situation

You are an intern in new startup. Your mentor needs you to develop some small applications to help the startup. 

The startup is growing fast, so there is a lot of new registrations. Each registration is pushed to Kafka, and there is a lot of small tasks to do.

## Architecture

Each team will develop an application that will:

* read Kafka messages,
* react to the message,
* write a Kafka message.

This is commonly known as **ETL**:

* **Extract** data,
* **Transform** data,
* **Load** data.

The application can be written in any language, but there will be support only in Java, Go, Python or Rust.

An online Python environment accessible from any browser can be used by clicking on the following link:

[![Open in GitPod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/PierreZ/kafka-tutorial).

## library

You can use those librairies to connect to Kafka:

* [Python client](https://kafka-python.readthedocs.io/en/master/)
* [Go client](https://github.com/Shopify/sarama)
* [Rust client](https://github.com/fede1024/rust-rdkafka)
* [Node client](https://www.npmjs.com/package/kafka-node)
* [Java client](https://search.maven.org/#artifactdetails%7Corg.apache.kafka%7Ckafka-clients%7C1.1.0%7Cjar)

You can now continue on [step-1](/kafka-tutorial/docs/step-1.html)!
