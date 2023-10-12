# monitoring

A set of small tools to collect and parse data from different sources.

## rmq

Collects RabbitMQ federation data and checks if all nodes upstreams are up and running.

Example:
```bash
RUST_LOG=info RMQ_HOSTS=<comma separated rmq node fqdns> RMQ_LOGIN=<login> RMQ_PASSWORD=<password> JUGGLER_URL=<juggler fqdn> cargo run
```

## zoo

Collects Zookeeper ensemble data, finds who is the leader and who are the followers.

Example:
```bash
RUST_LOG=info ZOO_HOSTS=<comma separated zookeeper node fqdns> JUGGLER_URL=<juggler fqdn> cargo running
```

### zoo1

Simplest version.

Pros:
  - no extra code

Cons:
  - error log messages are missing host information

### zoo2

Uses `anyhow` context to add host info to error log messages.

Pros:
  - host info included in error messages

Cons:
  - extra dependency
  - original error messages are lost

### zoo3

Uses custom struct wrapping `String` as an error. Enriches original error messages with the host info.

Pros:
  - host info included in error messages
  - original error messages

Cons:
  - A LOT of boilerplate code
  - have to manually handle errors and include host info

### zoo4

Uses custom struct wrapping `String` as an error. Enriches original error messages with the host info.
This time does this with `map_err` outside of `collect` function.

Pros:
  - host info included in error messages
  - original error messages
  - almost no extra code

### zoo5

Uses custom enum-based errors. Enriches original error messages with the host info.

Pros:
  - host info included in error messages
  - original error messages

Cons:
  - INSANE amount of boilerplate code
  - have to manually handle errors and include host info

### zoo6

Uses custom enum-based errors. Enriches original error messages with the host info.
[thiserror](https://crates.io/crates/thiserror) crate helps reduce the amount of boilerplate code.

Pros:
  - host info included in error messages
  - original error messages

Cons:
  - extra dependency
  - less boilerplate code but still a lot
  - have to manually handle errors and include host info

### Conclusion

It seems that `zoo4` version is the best one. Code is clean and concise.
