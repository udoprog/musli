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
- [Full features](#full-features) ([Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-full/report/))
- [Text-based formats](#text-based-formats) ([Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-text/report/))
- [Fewer features](#fewer-features) ([Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-fewer/report/))
- [Müsli vs rkyv](#müsli-vs-rkyv) ([Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/report/))
- [Müsli vs zerocopy](#müsli-vs-zerocopy) ([Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/report/))
- [Bitcode derive](#bitcode-derive) ([Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/report/))

Below you'll also find [Size comparisons](#size-comparisons).
# Full features

These frameworks provide a fair comparison against Müsli on various areas since
they support the same set of features in what types of data they can represent.

[Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-full/report/)

`primitives`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_full.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_full.svg">

`primpacked`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_full.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_full.svg">

`medium_enum`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_full.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_full.svg">

`large`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_full.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_full.svg">

`allocated`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_full.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_full.svg">


# Text-based formats

These are text-based formats, which support the full feature set of this test suite.

[Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-text/report/)

`primitives`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_text.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_text.svg">

`primpacked`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_text.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_text.svg">

`medium_enum`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_text.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_text.svg">

`large`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_text.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_text.svg">

`allocated`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_text.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_text.svg">


# Fewer features

> **Missing features:** `128`, `map`, `map-string-key`

This is a suite where support for 128-bit integers and maps are disabled.
Usually because the underlying framework lacks support for them.

[Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-fewer/report/)

`primitives`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_fewer.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_fewer.svg">

`primpacked`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_fewer.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_fewer.svg">

`medium_enum`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_fewer.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_fewer.svg">

`large`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_fewer.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_fewer.svg">

`allocated`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_fewer.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_fewer.svg">


# Müsli vs rkyv

> **Missing features:** `cstring`, `map`, `map-string-key`, `tuple`, `usize`

Comparison between [`musli-zerocopy`] and [`rkyv`].

Note that `musli-zerocopy` only supports the `primitives` benchmark.

[Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/report/)

`primitives`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_zerocopy-rkyv.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_zerocopy-rkyv.svg">

`primpacked`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_zerocopy-rkyv.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_zerocopy-rkyv.svg">


# Müsli vs zerocopy

Compares [`musli-zerocopy`] with [`zerocopy`].

Note that `zerocopy` only supports packed primitives, so we're only comparing with that suite.

[Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/report/)

`primpacked`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_zerocopy-zerocopy.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_zerocopy-zerocopy.svg">


# Bitcode derive

> **Missing features:** `cstring`

Uses a custom derive-based framework which does not support everything Müsli and serde does.

[Full criterion report](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/report/)

`primitives`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_bitcode-derive.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_bitcode-derive.svg">

`primpacked`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_bitcode-derive.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_bitcode-derive.svg">

`medium_enum`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_bitcode-derive.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_bitcode-derive.svg">

`large`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_bitcode-derive.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_bitcode-derive.svg">

`allocated`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_bitcode-derive.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_bitcode-derive.svg">


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

#### Full features

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| derive_bitcode | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 106, max: 106, stddev: 0">106.00 ± 0.00</a> | <a title="samples: 10, min: 20427, max: 42401, stddev: 7378.982528777257">33853.20 ± 7378.98</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 1, max: 122, stddev: 45.90805049226123">29.67 ± 45.91</a> |
| musli_descriptive | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 28754, max: 69169, stddev: 13606.13782232122">55043.60 ± 13606.14</a> | <a title="samples: 100, min: 952, max: 3859, stddev: 614.4581333174782">2444.68 ± 614.46</a> | <a title="samples: 500, min: 4, max: 191, stddev: 66.12034918238102">42.37 ± 66.12</a> |
| musli_storage | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 22439, max: 48857, stddev: 8964.162584982492">39176.50 ± 8964.16</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 133, stddev: 49.25819322711703">31.92 ± 49.26</a> |
| musli_storage_packed | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 20996, max: 45011, stddev: 8066.011728233477">35923.00 ± 8066.01</a> | <a title="samples: 100, min: 721, max: 2776, stddev: 443.9493911472342">1768.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 122, stddev: 44.125809952906295">29.34 ± 44.13</a> |
| musli_wire | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 27162, max: 64146, stddev: 12580.785293454459">51422.00 ± 12580.79</a> | <a title="samples: 100, min: 945, max: 3854, stddev: 615.0335824164403">2438.15 ± 615.03</a> | <a title="samples: 500, min: 3, max: 171, stddev: 59.694680634039706">38.35 ± 59.69</a> |

#### Text-based formats

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| derive_bitcode | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 106, max: 106, stddev: 0">106.00 ± 0.00</a> | <a title="samples: 10, min: 20427, max: 42401, stddev: 7378.982528777257">33853.20 ± 7378.98</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 1, max: 122, stddev: 45.90805049226123">29.67 ± 45.91</a> |
| musli_descriptive | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 28754, max: 69169, stddev: 13606.13782232122">55043.60 ± 13606.14</a> | <a title="samples: 100, min: 952, max: 3859, stddev: 614.4581333174782">2444.68 ± 614.46</a> | <a title="samples: 500, min: 4, max: 191, stddev: 66.12034918238102">42.37 ± 66.12</a> |
| musli_storage | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 22439, max: 48857, stddev: 8964.162584982492">39176.50 ± 8964.16</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 133, stddev: 49.25819322711703">31.92 ± 49.26</a> |
| musli_storage_packed | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 20996, max: 45011, stddev: 8066.011728233477">35923.00 ± 8066.01</a> | <a title="samples: 100, min: 721, max: 2776, stddev: 443.9493911472342">1768.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 122, stddev: 44.125809952906295">29.34 ± 44.13</a> |
| musli_wire | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 27162, max: 64146, stddev: 12580.785293454459">51422.00 ± 12580.79</a> | <a title="samples: 100, min: 945, max: 3854, stddev: 615.0335824164403">2438.15 ± 615.03</a> | <a title="samples: 500, min: 3, max: 171, stddev: 59.694680634039706">38.35 ± 59.69</a> |

#### Fewer features

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| derive_bitcode | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 106, max: 106, stddev: 0">106.00 ± 0.00</a> | <a title="samples: 10, min: 20427, max: 42401, stddev: 7378.982528777257">33853.20 ± 7378.98</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 1, max: 122, stddev: 45.90805049226123">29.67 ± 45.91</a> |
| musli_descriptive | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 28754, max: 69169, stddev: 13606.13782232122">55043.60 ± 13606.14</a> | <a title="samples: 100, min: 952, max: 3859, stddev: 614.4581333174782">2444.68 ± 614.46</a> | <a title="samples: 500, min: 4, max: 191, stddev: 66.12034918238102">42.37 ± 66.12</a> |
| musli_storage | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 22439, max: 48857, stddev: 8964.162584982492">39176.50 ± 8964.16</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 133, stddev: 49.25819322711703">31.92 ± 49.26</a> |
| musli_storage_packed | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 20996, max: 45011, stddev: 8066.011728233477">35923.00 ± 8066.01</a> | <a title="samples: 100, min: 721, max: 2776, stddev: 443.9493911472342">1768.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 122, stddev: 44.125809952906295">29.34 ± 44.13</a> |
| musli_wire | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 27162, max: 64146, stddev: 12580.785293454459">51422.00 ± 12580.79</a> | <a title="samples: 100, min: 945, max: 3854, stddev: 615.0335824164403">2438.15 ± 615.03</a> | <a title="samples: 500, min: 3, max: 171, stddev: 59.694680634039706">38.35 ± 59.69</a> |

#### Müsli vs rkyv

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| derive_bitcode | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 106, max: 106, stddev: 0">106.00 ± 0.00</a> | <a title="samples: 10, min: 20427, max: 42401, stddev: 7378.982528777257">33853.20 ± 7378.98</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 1, max: 122, stddev: 45.90805049226123">29.67 ± 45.91</a> |
| musli_descriptive | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 28754, max: 69169, stddev: 13606.13782232122">55043.60 ± 13606.14</a> | <a title="samples: 100, min: 952, max: 3859, stddev: 614.4581333174782">2444.68 ± 614.46</a> | <a title="samples: 500, min: 4, max: 191, stddev: 66.12034918238102">42.37 ± 66.12</a> |
| musli_storage | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 22439, max: 48857, stddev: 8964.162584982492">39176.50 ± 8964.16</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 133, stddev: 49.25819322711703">31.92 ± 49.26</a> |
| musli_storage_packed | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 20996, max: 45011, stddev: 8066.011728233477">35923.00 ± 8066.01</a> | <a title="samples: 100, min: 721, max: 2776, stddev: 443.9493911472342">1768.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 122, stddev: 44.125809952906295">29.34 ± 44.13</a> |
| musli_wire | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 27162, max: 64146, stddev: 12580.785293454459">51422.00 ± 12580.79</a> | <a title="samples: 100, min: 945, max: 3854, stddev: 615.0335824164403">2438.15 ± 615.03</a> | <a title="samples: 500, min: 3, max: 171, stddev: 59.694680634039706">38.35 ± 59.69</a> |

#### Müsli vs zerocopy

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| derive_bitcode | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 106, max: 106, stddev: 0">106.00 ± 0.00</a> | <a title="samples: 10, min: 20427, max: 42401, stddev: 7378.982528777257">33853.20 ± 7378.98</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 1, max: 122, stddev: 45.90805049226123">29.67 ± 45.91</a> |
| musli_descriptive | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 28754, max: 69169, stddev: 13606.13782232122">55043.60 ± 13606.14</a> | <a title="samples: 100, min: 952, max: 3859, stddev: 614.4581333174782">2444.68 ± 614.46</a> | <a title="samples: 500, min: 4, max: 191, stddev: 66.12034918238102">42.37 ± 66.12</a> |
| musli_storage | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 22439, max: 48857, stddev: 8964.162584982492">39176.50 ± 8964.16</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 133, stddev: 49.25819322711703">31.92 ± 49.26</a> |
| musli_storage_packed | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 20996, max: 45011, stddev: 8066.011728233477">35923.00 ± 8066.01</a> | <a title="samples: 100, min: 721, max: 2776, stddev: 443.9493911472342">1768.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 122, stddev: 44.125809952906295">29.34 ± 44.13</a> |
| musli_wire | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 27162, max: 64146, stddev: 12580.785293454459">51422.00 ± 12580.79</a> | <a title="samples: 100, min: 945, max: 3854, stddev: 615.0335824164403">2438.15 ± 615.03</a> | <a title="samples: 500, min: 3, max: 171, stddev: 59.694680634039706">38.35 ± 59.69</a> |

#### Bitcode derive

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| derive_bitcode | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 106, max: 106, stddev: 0">106.00 ± 0.00</a> | <a title="samples: 10, min: 20427, max: 42401, stddev: 7378.982528777257">33853.20 ± 7378.98</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 1, max: 122, stddev: 45.90805049226123">29.67 ± 45.91</a> |
| musli_descriptive | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 28754, max: 69169, stddev: 13606.13782232122">55043.60 ± 13606.14</a> | <a title="samples: 100, min: 952, max: 3859, stddev: 614.4581333174782">2444.68 ± 614.46</a> | <a title="samples: 500, min: 4, max: 191, stddev: 66.12034918238102">42.37 ± 66.12</a> |
| musli_storage | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 22439, max: 48857, stddev: 8964.162584982492">39176.50 ± 8964.16</a> | <a title="samples: 100, min: 728, max: 2783, stddev: 443.9493911472342">1775.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 133, stddev: 49.25819322711703">31.92 ± 49.26</a> |
| musli_storage_packed | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 20996, max: 45011, stddev: 8066.011728233477">35923.00 ± 8066.01</a> | <a title="samples: 100, min: 721, max: 2776, stddev: 443.9493911472342">1768.91 ± 443.95</a> | <a title="samples: 500, min: 2, max: 122, stddev: 44.125809952906295">29.34 ± 44.13</a> |
| musli_wire | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 27162, max: 64146, stddev: 12580.785293454459">51422.00 ± 12580.79</a> | <a title="samples: 100, min: 945, max: 3854, stddev: 615.0335824164403">2438.15 ± 615.03</a> | <a title="samples: 500, min: 3, max: 171, stddev: 59.694680634039706">38.35 ± 59.69</a> |

[`rkyv`]: https://docs.rs/rkyv
[`zerocopy`]: https://docs.rs/zerocopy
[`musli-zerocopy`]: https://docs.rs/musli-zerocopy
