# Tools used with Müsli development

Note that the `--report <id>` argument is available to filter out to use
parameters for one specific report (by `id`).

```sh
cargo run -p tools -- bench --report full
```

#### Generating report

This obviously takes a long time, but will walk through all feature combinations
and generate a report:

```sh
cargo run -p tools -- report --release
```

> If you want the faster version for testing, add `--quick`.

#### Running benchmarks

This will run all benchmarks, one for each report.

```sh
cargo run -p tools -- bench
```


#### Running clippy

This will run clippy and sanity check the configuration, one for each report.

```sh
cargo run -p tools -- clippy
```
