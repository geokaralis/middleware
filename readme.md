# ECR Middleware

A service for handling communication between ECR machines and POS devices.

## Flow

```mermaid
sequenceDiagram
    participant ECR
    participant Middleware as ECR Middleware
    participant NATS
    participant POS

    ECR->>Middleware: Send message via TCP
    Middleware->>NATS: Forward message via jetstream
    NATS->>POS: Forward message via jetstream
    POS-->>NATS: Send response via jetstream
    NATS-->>Middleware: Forward response via jetstream
    Middleware-->>ECR: Send response via TCP
```

## Prerequisites

1. Install [`rust`]
2. Install [`docker`]

## Dev

To run the application in development mode, use the following command:

```
RUST_LOG=debug cargo run
```

## Docker

To start the application using Docker Compose, use the command:

```
docker compose up -d
```

To stop and remove the containers, use the command:

```
docker compose down
```

## Working with the examples

To run an example pos client for receiving messages, use the command:

```
cargo run --example pos
```

Similarly, to run an example ecr client for sending messages, use the command:

```
cargo run --example ecr
```

## Running tests

Some tests rely on [`testcontainers`] make sure docker is running.

To run all tests, use the following command:

```
cargo test
```

### Running a specific test

To run a specific test (e.g., a test named multiple), use the command:

```
cargo test --test multiple
```

## Architecture

```mermaid
graph TD
    subgraph K8s Cluster
        NATS <--> Middleware[ECR Middleware]
    end
        ECR[ECR] <--> Middleware
        POS[POS] <--> NATS
```

[`rust`]: https://www.rust-lang.org/tools/install
[`docker`]: https://docs.docker.com/engine/install/
[`testcontainers`]: https://docs.rs/crate/testcontainers/latest
