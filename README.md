# Alma

```shell
# node 1
$ cargo run -- --port 8765 --remote-address 127.0.0.1:8766
# or
$ cargo run -- --config conf-a.toml

# node 2
$ cargo run -- --port 8766 --remote-address 127.0.0.1:8765
# or
$cargo run -- --config conf-b.toml
```