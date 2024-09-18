# Alma

```shell
# answer
$ cargo run -- --mode answer --port 8765 --remote-address 127.0.0.1:8766
# or
$ cargo run -- --config conf-a.toml

# offer
$ cargo run -- --mode offer --port 8766 --remote-address 127.0.0.1:8765
# or
$cargo run -- --config conf-b.toml
```