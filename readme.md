# Redis Streams Exporter

Prometheus exporter for Redis Streams by the redis command `xinfo`.

And this project is written by rust, it provide a painless, standalone, economical, simple monitor for the poor men who choice the lightweight queues by redis like me.

## Installation and Usage

### Binary

[Download from github](https://github.com/cnzx219/redis-streams-expoter/releases)

Then in terminal:

```bash
./redis-streams-exporter '--key=test-streams1;test-streams2;test-streams3' --redis=redis://127.0.0.1/0 --bind=127.0.0.1:9219
```

Or simplify:

```bash
./redis-streams-exporter --key=test-streams1
```

### Docker

Coming soon.

### From Source

```bash
cargo build --release
```

## Configuration

All configuration is handled via command arguments.

| Environment Variable | Required                              | Description                                                                                                              |
|----------------------|---------------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| `key`                | Yes                                   | The name of the Stream(s) to monitor. Can specify multiple keys by delimiting with a semi-colon, e.g. `key-one;key-two`. |
| `redis`              | No, defaults to "redis://127.0.0.1/0" | Redis Url                                                                                                                |
| `bind`               | No, defaults to "127.0.0.1:9219"      | The ip and port is binded by this exporter.                                                                              |
| `calc-idle`          | No, defaults to "false"               | The time (secounds) since the last message.                                                                              |

## Metrics

Metrics will be exported at varying levels of granularity.

### Stream

```
# HELP redis_stream_length Number of messages in the stream
# TYPE redis_stream_length gauge
redis_stream_length{stream="my-stream-key"} 24601

# HELP redis_stream_earliest_id The epoch timestamp of the earliest message on the stream
# TYPE redis_stream_earliest_id gauge
redis_stream_earliest_id{stream="my-stream-key"} 1597104418874

# HELP redis_stream_latest_id The epoch timestamp of the latest message on the stream
# TYPE redis_stream_latest_id gauge
redis_stream_latest_id{stream="my-stream-key"} 1597152683722

# HELP redis_stream_consumer_groups_total Number of consumer groups for the stream
# TYPE redis_stream_consumer_groups_total gauge
redis_stream_consumer_groups_total{stream="my-stream-key"} 3
```

### Consumer Group
```
# HELP redis_stream_consumer_group_last_delivered_id The epoch timestamp of the last delivered message
# TYPE redis_stream_consumer_group_last_delivered_id gauge
redis_stream_consumer_group_last_delivered_id{stream="my-stream-key",group="group-a"} 1597152683722
redis_stream_consumer_group_last_delivered_id{stream="my-stream-key",group="group-b"} 1597152683722
redis_stream_consumer_group_last_delivered_id{stream="my-stream-key",group="group-c"} 1597152683722

# HELP redis_stream_consumer_group_pending_messages_total Number of pending messages for the group
# TYPE redis_stream_consumer_group_pending_messages_total gauge
redis_stream_consumer_group_pending_messages_total{stream="my-stream-key",group="group-a"} 0
redis_stream_consumer_group_pending_messages_total{stream="my-stream-key",group="group-b"} 0
redis_stream_consumer_group_pending_messages_total{stream="my-stream-key",group="group-c"} 0

# HELP redis_stream_consumer_group_consumers_total Number of consumers in the group
# TYPE redis_stream_consumer_group_consumers_total gauge
redis_stream_consumer_group_consumers_total{stream="my-stream-key",group="group-a"} 1
redis_stream_consumer_group_consumers_total{stream="my-stream-key",group="group-b"} 1
redis_stream_consumer_group_consumers_total{stream="my-stream-key",group="group-c"} 1
```

### Consumer
```
# HELP redis_stream_consumer_pending_messages_total Number of pending messages for the consumer
# TYPE redis_stream_consumer_pending_messages_total gauge
redis_stream_consumer_pending_messages_total{stream="my-stream-key",group="group-a",consumer="dhHXcC1E3"} 0
redis_stream_consumer_pending_messages_total{stream="my-stream-key",group="group-b",consumer="UgfoRw0ew"} 0
redis_stream_consumer_pending_messages_total{stream="my-stream-key",group="group-c",consumer="4gXR54IYg"} 0

# HELP redis_stream_consumer_idle_time_seconds The amount of time for which the consumer has been idle
# TYPE redis_stream_consumer_idle_time_seconds gauge
redis_stream_consumer_idle_time_seconds{stream="my-stream-key",group="group-a",consumer="dhHXcC1E3"} 77.063
redis_stream_consumer_idle_time_seconds{stream="my-stream-key",group="group-b",consumer="UgfoRw0ew"} 77.064
redis_stream_consumer_idle_time_seconds{stream="my-stream-key",group="group-c",consumer="4gXR54IYg"} 77.064
```

##  Refer to

https://github.com/chrnola/redis-streams-exporter
