# Benchmarks and size comparisons

> The following are the results of preliminary benchmarking and should be
> taken with a big grain of 🧂.

Summary of the different kinds of benchmarks we support.

- `primitives` which is a small object containing one of each primitive type and a string and a byte array.
- `primpacked` Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries.
- `medium_enum` A moderately sized enum with many field variations.
- `large` A really big and complex struct.
- `allocated` A sparse struct which contains fairly plain allocated data like strings and vectors.

The following are one section for each kind of benchmark we perform. They range from "Full features" to more specialized ones like zerocopy comparisons.

Below you'll also find [Size comparisons](#size-comparisons).
# Size comparisons

This is not yet an area which has received much focus, but because people are bound to ask the following section performs a raw size comparison between different formats.
Each test suite serializes a collection of values, which have all been randomly populated.
- which is a small object containing one of each primitive type and a string and a byte array. (`primitives`)
- Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries. (`primpacked`)
- A moderately sized enum with many field variations. (`medium_enum`)
- A really big and complex struct. (`large`)
- A sparse struct which contains fairly plain allocated data like strings and vectors. (`allocated`)

> **Note** so far these are all synthetic examples. Real world data is
> rarely *this* random. But hopefully it should give an idea of the extreme
> ranges.

[`rkyv`]: https://docs.rs/rkyv
[`zerocopy`]: https://docs.rs/zerocopy
[`musli-zerocopy`]: https://docs.rs/musli-zerocopy
