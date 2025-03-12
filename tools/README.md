# Tools used with Müsli development

Note that the `--report <id>` argument is available to filter out to use parameters for one specific report:

```sh
cargo run -- --report full bench
```

#### Generating benchmarks

This obviously takes a long time, but will walk through all feature
combinations:

```sh
cargo run -- report --bench
```

#### Running benchmarks

This will run all benchmarks, one for each report.

```sh
cargo run -p tools -- bench
```


#### Running clippy

This will run clippy ant sanity check the configuration, one for each report.

```sh
cargo run -p tools -- bench
```
