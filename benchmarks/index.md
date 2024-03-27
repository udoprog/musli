# Benchmarks and size comparisons

> The following are the results of preliminary benchmarking and should be
> taken with a big grain of 🧂.

Identifiers which are used in tests:

- `dec` - Decode a type.
- `enc` - Encode a type.
- `primitives` - A small object containing one of each primitive type and a string and a byte array.
- `primpacked` - Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries.
- `medium_enum` - A moderately sized enum with every kind of supported variant.
- `large` - A really big and complex struct.
- `allocated` - A sparse struct which contains fairly plain allocated data like strings and vectors.

The following are one section for each kind of benchmark we perform. They range from "Full features" to more specialized ones like zerocopy comparisons.
- [**Full features**](#full-features) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/report/), [Sizes](#full-features-sizes))
- [**Text-based formats**](#text-based-formats) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/report/), [Sizes](#text-based-formats-sizes))
- [**Fewer features**](#fewer-features) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/report/), [Sizes](#fewer-features-sizes))
- [**Müsli vs rkyv**](#müsli-vs-rkyv) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/report/), [Sizes](#müsli-vs-rkyv-sizes))
- [**Müsli vs zerocopy**](#müsli-vs-zerocopy) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/report/), [Sizes](#müsli-vs-zerocopy-sizes))
- [**Bitcode derive**](#bitcode-derive) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/report/), [Sizes](#bitcode-derive-sizes))
- [**BSON**](#bson) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/report/), [Sizes](#bson-sizes))

Below you'll also find [size comparisons](#size-comparisons).
### Full features

These frameworks provide a fair comparison against Müsli on various areas since
they support the same set of features in what types of data they can represent.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/report/)
* [Sizes](#full-features-sizes)

<table>
<tr>
<th colspan="3">
<code>full/dec/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primitives/musli_descriptive` | **987.35ns** ± 0.74ns | 986.10ns &mdash; 988.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **427.30ns** ± 0.42ns | 426.54ns &mdash; 428.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **93.11ns** ± 0.09ns | 92.95ns &mdash; 93.29ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_value`[^musli_value] | **395.31ns** ± 0.28ns | 394.86ns &mdash; 395.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **810.74ns** ± 0.57ns | 809.76ns &mdash; 811.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_wire/report/) |
| `dec/primitives/postcard` | **256.20ns** ± 0.18ns | 255.91ns &mdash; 256.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/postcard/report/) |
| `dec/primitives/serde_bincode` | **80.02ns** ± 0.08ns | 79.87ns &mdash; 80.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bincode/report/) |
| `dec/primitives/serde_bitcode` | **1.29μs** ± 0.97ns | 1.29μs &mdash; 1.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bitcode/report/) |
| `dec/primitives/serde_rmp` | **327.66ns** ± 0.22ns | 327.30ns &mdash; 328.16ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_rmp/report/) |

<table>
<tr>
<th colspan="3">
<code>full/enc/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primitives/musli_descriptive` | **866.52ns** ± 1.03ns | 864.93ns &mdash; 868.87ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **291.75ns** ± 0.25ns | 291.31ns &mdash; 292.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **132.40ns** ± 0.08ns | 132.27ns &mdash; 132.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.03μs** ± 0.75ns | 1.03μs &mdash; 1.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **764.95ns** ± 0.72ns | 763.71ns &mdash; 766.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_wire/report/) |
| `enc/primitives/postcard` | **434.49ns** ± 0.27ns | 434.06ns &mdash; 435.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/postcard/report/) |
| `enc/primitives/serde_bincode` | **104.56ns** ± 0.11ns | 104.42ns &mdash; 104.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bincode/report/) |
| `enc/primitives/serde_bitcode` | **4.15μs** ± 4.90ns | 4.15μs &mdash; 4.17μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bitcode/report/) |
| `enc/primitives/serde_rmp` | **226.29ns** ± 0.20ns | 225.93ns &mdash; 226.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_rmp/report/) |


<table>
<tr>
<th colspan="3">
<code>full/dec/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primpacked/musli_descriptive` | **976.04ns** ± 0.71ns | 974.85ns &mdash; 977.60ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **488.79ns** ± 0.66ns | 487.62ns &mdash; 490.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **488.54ns** ± 0.42ns | 487.79ns &mdash; 489.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **462.10ns** ± 0.34ns | 461.60ns &mdash; 462.87ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **829.16ns** ± 0.62ns | 828.10ns &mdash; 830.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/postcard` | **249.88ns** ± 0.16ns | 249.63ns &mdash; 250.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/postcard/report/) |
| `dec/primpacked/serde_bincode` | **59.13ns** ± 0.05ns | 59.06ns &mdash; 59.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_bincode/report/) |
| `dec/primpacked/serde_bitcode` | **1.55μs** ± 1.25ns | 1.55μs &mdash; 1.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_bitcode/report/) |
| `dec/primpacked/serde_rmp` | **427.54ns** ± 0.32ns | 427.00ns &mdash; 428.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_rmp/report/) |

<table>
<tr>
<th colspan="3">
<code>full/enc/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primpacked/musli_descriptive` | **872.05ns** ± 0.72ns | 870.77ns &mdash; 873.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **231.91ns** ± 0.25ns | 231.58ns &mdash; 232.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **232.07ns** ± 0.17ns | 231.77ns &mdash; 232.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **1.45μs** ± 1.24ns | 1.45μs &mdash; 1.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **737.05ns** ± 0.77ns | 735.59ns &mdash; 738.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/postcard` | **425.08ns** ± 0.49ns | 424.33ns &mdash; 426.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/postcard/report/) |
| `enc/primpacked/serde_bincode` | **124.58ns** ± 0.09ns | 124.43ns &mdash; 124.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_bincode/report/) |
| `enc/primpacked/serde_bitcode` | **4.67μs** ± 6.05ns | 4.66μs &mdash; 4.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_bitcode/report/) |
| `enc/primpacked/serde_rmp` | **253.75ns** ± 0.19ns | 253.44ns &mdash; 254.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_rmp/report/) |


<table>
<tr>
<th colspan="3">
<code>full/dec/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/medium_enum/musli_descriptive` | **2.27μs** ± 2.72ns | 2.27μs &mdash; 2.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.18μs** ± 1.06ns | 1.18μs &mdash; 1.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **841.56ns** ± 0.53ns | 840.65ns &mdash; 842.71ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **1.02μs** ± 0.78ns | 1.02μs &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **1.84μs** ± 1.72ns | 1.83μs &mdash; 1.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/postcard` | **1.19μs** ± 0.93ns | 1.19μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/postcard/report/) |
| `dec/medium_enum/serde_bincode` | **865.94ns** ± 0.75ns | 864.81ns &mdash; 867.64ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_bincode/report/) |
| `dec/medium_enum/serde_bitcode` | **9.19μs** ± 7.68ns | 9.18μs &mdash; 9.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_bitcode/report/) |
| `dec/medium_enum/serde_rmp` | **2.40μs** ± 2.60ns | 2.39μs &mdash; 2.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_rmp/report/) |

<table>
<tr>
<th colspan="3">
<code>full/enc/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/medium_enum/musli_descriptive` | **1.59μs** ± 1.24ns | 1.59μs &mdash; 1.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **666.40ns** ± 0.71ns | 665.16ns &mdash; 667.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **500.13ns** ± 0.37ns | 499.53ns &mdash; 500.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **3.21μs** ± 2.95ns | 3.21μs &mdash; 3.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **1.40μs** ± 1.07ns | 1.40μs &mdash; 1.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/postcard` | **915.61ns** ± 0.94ns | 913.87ns &mdash; 917.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/postcard/report/) |
| `enc/medium_enum/serde_bincode` | **297.82ns** ± 0.23ns | 297.50ns &mdash; 298.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_bincode/report/) |
| `enc/medium_enum/serde_bitcode` | **13.53μs** ± 10.26ns | 13.51μs &mdash; 13.55μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_bitcode/report/) |
| `enc/medium_enum/serde_rmp` | **753.63ns** ± 1.12ns | 751.53ns &mdash; 755.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_rmp/report/) |


<table>
<tr>
<th colspan="3">
<code>full/dec/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/large/musli_descriptive` | **273.87μs** ± 257.28ns | 273.45μs &mdash; 274.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **88.65μs** ± 75.53ns | 88.55μs &mdash; 88.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **46.60μs** ± 54.82ns | 46.51μs &mdash; 46.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_value`[^musli_value] | **118.19μs** ± 354.91ns | 117.56μs &mdash; 118.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **258.39μs** ± 323.17ns | 257.94μs &mdash; 259.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_wire/report/) |
| `dec/large/postcard` | **86.72μs** ± 120.57ns | 86.53μs &mdash; 86.99μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/postcard/report/) |
| `dec/large/serde_bincode` | **59.81μs** ± 60.72ns | 59.72μs &mdash; 59.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bincode/report/) |
| `dec/large/serde_bitcode` | **98.32μs** ± 127.63ns | 98.08μs &mdash; 98.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bitcode/report/) |
| `dec/large/serde_rmp` | **231.02μs** ± 307.99ns | 230.55μs &mdash; 231.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_rmp/report/) |

<table>
<tr>
<th colspan="3">
<code>full/enc/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/large/musli_descriptive` | **187.44μs** ± 193.82ns | 187.14μs &mdash; 187.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **63.99μs** ± 52.61ns | 63.89μs &mdash; 64.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **29.66μs** ± 19.29ns | 29.63μs &mdash; 29.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_value`[^musli_value] | **709.06μs** ± 1.10μs | 707.52μs &mdash; 711.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **159.80μs** ± 145.78ns | 159.58μs &mdash; 160.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_wire/report/) |
| `enc/large/postcard` | **112.61μs** ± 234.80ns | 112.23μs &mdash; 113.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/postcard/report/) |
| `enc/large/serde_bincode` | **40.23μs** ± 32.08ns | 40.18μs &mdash; 40.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bincode/report/) |
| `enc/large/serde_bitcode` | **117.70μs** ± 98.32ns | 117.54μs &mdash; 117.92μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bitcode/report/) |
| `enc/large/serde_rmp` | **128.16μs** ± 153.02ns | 127.92μs &mdash; 128.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_rmp/report/) |


<table>
<tr>
<th colspan="3">
<code>full/dec/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/allocated/musli_descriptive` | **3.48μs** ± 3.35ns | 3.47μs &mdash; 3.48μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **3.05μs** ± 3.79ns | 3.04μs &mdash; 3.06μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.57μs** ± 2.33ns | 2.57μs &mdash; 2.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_value`[^musli_value] | **2.17μs** ± 2.14ns | 2.17μs &mdash; 2.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **3.67μs** ± 4.11ns | 3.66μs &mdash; 3.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_wire/report/) |
| `dec/allocated/postcard` | **3.42μs** ± 3.74ns | 3.41μs &mdash; 3.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/postcard/report/) |
| `dec/allocated/serde_bincode` | **3.18μs** ± 4.51ns | 3.17μs &mdash; 3.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bincode/report/) |
| `dec/allocated/serde_bitcode` | **5.72μs** ± 5.55ns | 5.71μs &mdash; 5.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bitcode/report/) |
| `dec/allocated/serde_rmp` | **4.34μs** ± 3.55ns | 4.33μs &mdash; 4.34μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_rmp/report/) |

<table>
<tr>
<th colspan="3">
<code>full/enc/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/allocated/musli_descriptive` | **823.57ns** ± 0.58ns | 822.66ns &mdash; 824.86ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **396.47ns** ± 0.41ns | 395.83ns &mdash; 397.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **320.81ns** ± 0.27ns | 320.36ns &mdash; 321.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_value`[^musli_value] | **2.38μs** ± 2.35ns | 2.38μs &mdash; 2.39μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **756.17ns** ± 0.62ns | 755.01ns &mdash; 757.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_wire/report/) |
| `enc/allocated/postcard` | **1.21μs** ± 1.09ns | 1.20μs &mdash; 1.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/postcard/report/) |
| `enc/allocated/serde_bincode` | **315.46ns** ± 0.65ns | 314.43ns &mdash; 316.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bincode/report/) |
| `enc/allocated/serde_bitcode` | **8.23μs** ± 7.18ns | 8.21μs &mdash; 8.24μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bitcode/report/) |
| `enc/allocated/serde_rmp` | **764.84ns** ± 0.58ns | 763.86ns &mdash; 766.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_rmp/report/) |



### Text-based formats

These are text-based formats, which support the full feature set of this test suite.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/report/)
* [Sizes](#text-based-formats-sizes)

<table>
<tr>
<th colspan="3">
<code>text/dec/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primitives/musli_json` | **3.24μs** ± 2.66ns | 3.24μs &mdash; 3.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **4.50μs** ± 4.03ns | 4.49μs &mdash; 4.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>text/enc/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primitives/musli_json` | **761.75ns** ± 0.60ns | 760.85ns &mdash; 763.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **1.30μs** ± 1.56ns | 1.30μs &mdash; 1.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>text/dec/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primpacked/musli_json` | **3.95μs** ± 3.58ns | 3.94μs &mdash; 3.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/musli_json/report/) |
| `dec/primpacked/serde_json` | **4.72μs** ± 5.13ns | 4.71μs &mdash; 4.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>text/enc/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primpacked/musli_json` | **828.99ns** ± 0.74ns | 827.78ns &mdash; 830.62ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/musli_json/report/) |
| `enc/primpacked/serde_json` | **1.39μs** ± 1.19ns | 1.39μs &mdash; 1.39μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>text/dec/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/medium_enum/musli_json` | **8.73μs** ± 12.07ns | 8.71μs &mdash; 8.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/musli_json/report/) |
| `dec/medium_enum/serde_json` | **8.64μs** ± 7.01ns | 8.63μs &mdash; 8.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>text/enc/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/medium_enum/musli_json` | **1.86μs** ± 1.70ns | 1.85μs &mdash; 1.86μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/musli_json/report/) |
| `enc/medium_enum/serde_json` | **2.59μs** ± 2.25ns | 2.58μs &mdash; 2.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>text/dec/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/large/musli_json` | **1.01ms** ± 1.19μs | 1.01ms &mdash; 1.01ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **758.72μs** ± 593.30ns | 757.82μs &mdash; 760.06μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>text/enc/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/large/musli_json` | **246.15μs** ± 188.86ns | 245.89μs &mdash; 246.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **305.17μs** ± 389.84ns | 304.58μs &mdash; 306.06μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>text/dec/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/allocated/musli_json` | **9.27μs** ± 10.87ns | 9.25μs &mdash; 9.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **8.71μs** ± 8.51ns | 8.70μs &mdash; 8.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>text/enc/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/allocated/musli_json` | **2.28μs** ± 1.91ns | 2.28μs &mdash; 2.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **2.53μs** ± 2.29ns | 2.52μs &mdash; 2.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/serde_json/report/) |



### Fewer features

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `map` - Maps are not supported.

This is a suite where support for 128-bit integers and maps are disabled.
Usually because the underlying framework lacks support for them.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/report/)
* [Sizes](#fewer-features-sizes)

<table>
<tr>
<th colspan="3">
<code>fewer/dec/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primitives/musli_descriptive` | **778.81ns** ± 0.67ns | 777.64ns &mdash; 780.25ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **374.55ns** ± 0.56ns | 373.66ns &mdash; 375.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **88.33ns** ± 0.07ns | 88.19ns &mdash; 88.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_value`[^musli_value] | **347.65ns** ± 0.57ns | 346.54ns &mdash; 348.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **671.31ns** ± 0.50ns | 670.48ns &mdash; 672.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_wire/report/) |
| `dec/primitives/serde_cbor` | **1.53μs** ± 3.86ns | 1.53μs &mdash; 1.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/serde_cbor/report/) |

<table>
<tr>
<th colspan="3">
<code>fewer/enc/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primitives/musli_descriptive` | **547.55ns** ± 0.59ns | 546.43ns &mdash; 548.75ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **261.35ns** ± 0.29ns | 260.93ns &mdash; 262.01ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **124.50ns** ± 0.10ns | 124.33ns &mdash; 124.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.02μs** ± 0.90ns | 1.02μs &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **420.67ns** ± 0.41ns | 419.91ns &mdash; 421.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_wire/report/) |
| `enc/primitives/serde_cbor` | **415.63ns** ± 0.47ns | 414.99ns &mdash; 416.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/serde_cbor/report/) |


<table>
<tr>
<th colspan="3">
<code>fewer/dec/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primpacked/musli_descriptive` | **801.35ns** ± 0.52ns | 800.50ns &mdash; 802.51ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **399.25ns** ± 0.37ns | 398.60ns &mdash; 400.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **399.02ns** ± 0.33ns | 398.48ns &mdash; 399.74ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **398.20ns** ± 0.29ns | 397.71ns &mdash; 398.85ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **721.90ns** ± 0.54ns | 721.06ns &mdash; 723.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/serde_cbor` | **1.67μs** ± 3.22ns | 1.66μs &mdash; 1.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/serde_cbor/report/) |

<table>
<tr>
<th colspan="3">
<code>fewer/enc/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primpacked/musli_descriptive` | **549.02ns** ± 0.58ns | 547.92ns &mdash; 550.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **216.38ns** ± 0.15ns | 216.14ns &mdash; 216.71ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **216.29ns** ± 0.15ns | 216.05ns &mdash; 216.62ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **1.22μs** ± 0.90ns | 1.22μs &mdash; 1.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **432.28ns** ± 0.39ns | 431.59ns &mdash; 433.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/serde_cbor` | **484.65ns** ± 0.56ns | 483.65ns &mdash; 485.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/serde_cbor/report/) |


<table>
<tr>
<th colspan="3">
<code>fewer/dec/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/medium_enum/musli_descriptive` | **2.00μs** ± 2.22ns | 2.00μs &mdash; 2.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.11μs** ± 0.69ns | 1.11μs &mdash; 1.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **852.11ns** ± 0.62ns | 851.08ns &mdash; 853.50ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **969.51ns** ± 1.11ns | 967.62ns &mdash; 971.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **1.63μs** ± 1.33ns | 1.62μs &mdash; 1.63μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/serde_cbor` | **4.22μs** ± 5.06ns | 4.21μs &mdash; 4.23μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/serde_cbor/report/) |

<table>
<tr>
<th colspan="3">
<code>fewer/enc/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/medium_enum/musli_descriptive` | **1.29μs** ± 1.07ns | 1.29μs &mdash; 1.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **650.10ns** ± 0.88ns | 648.84ns &mdash; 652.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **510.84ns** ± 0.51ns | 509.92ns &mdash; 511.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **3.21μs** ± 2.99ns | 3.21μs &mdash; 3.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **1.05μs** ± 0.89ns | 1.05μs &mdash; 1.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/serde_cbor` | **975.60ns** ± 1.35ns | 973.36ns &mdash; 978.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/serde_cbor/report/) |


<table>
<tr>
<th colspan="3">
<code>fewer/dec/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/large/musli_descriptive` | **302.12μs** ± 150.58ns | 301.87μs &mdash; 302.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **97.67μs** ± 69.94ns | 97.55μs &mdash; 97.82μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **39.70μs** ± 28.08ns | 39.65μs &mdash; 39.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_value`[^musli_value] | **129.57μs** ± 377.29ns | 128.94μs &mdash; 130.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **273.02μs** ± 246.72ns | 272.58μs &mdash; 273.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_wire/report/) |
| `dec/large/serde_cbor` | **500.50μs** ± 813.65ns | 499.32μs &mdash; 502.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/serde_cbor/report/) |

<table>
<tr>
<th colspan="3">
<code>fewer/enc/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/large/musli_descriptive` | **204.66μs** ± 181.63ns | 204.40μs &mdash; 205.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **76.30μs** ± 80.56ns | 76.17μs &mdash; 76.48μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **35.90μs** ± 23.43ns | 35.87μs &mdash; 35.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_value`[^musli_value] | **760.74μs** ± 699.21ns | 759.64μs &mdash; 762.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **172.53μs** ± 120.83ns | 172.33μs &mdash; 172.80μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_wire/report/) |
| `enc/large/serde_cbor` | **165.38μs** ± 124.55ns | 165.19μs &mdash; 165.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/serde_cbor/report/) |


<table>
<tr>
<th colspan="3">
<code>fewer/dec/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/allocated/musli_descriptive` | **2.42μs** ± 2.87ns | 2.41μs &mdash; 2.42μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **2.08μs** ± 2.86ns | 2.07μs &mdash; 2.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **1.83μs** ± 1.62ns | 1.82μs &mdash; 1.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_value`[^musli_value] | **1.49μs** ± 1.35ns | 1.49μs &mdash; 1.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **2.49μs** ± 2.84ns | 2.48μs &mdash; 2.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_wire/report/) |
| `dec/allocated/serde_cbor` | **5.05μs** ± 4.00ns | 5.05μs &mdash; 5.06μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/serde_cbor/report/) |

<table>
<tr>
<th colspan="3">
<code>fewer/enc/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/allocated/musli_descriptive` | **622.80ns** ± 1.06ns | 620.98ns &mdash; 625.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **311.08ns** ± 0.23ns | 310.71ns &mdash; 311.60ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **246.78ns** ± 0.18ns | 246.47ns &mdash; 247.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_value`[^musli_value] | **1.95μs** ± 2.39ns | 1.94μs &mdash; 1.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **521.33ns** ± 0.40ns | 520.60ns &mdash; 522.16ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_wire/report/) |
| `enc/allocated/serde_cbor` | **613.58ns** ± 0.40ns | 612.92ns &mdash; 614.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/serde_cbor/report/) |



### Müsli vs rkyv

> **Missing features:**
> - `cstring` - `CString`'s are not supported.
> - `string-key` - Maps with strings as keys like `HashMap<String, T>` are not supported.
> - `string-set` - String sets like `HashSet<String>` are not supported.
> - `tuple` - Tuples like `(u32, u32)` are not supported.
> - `usize` - `usize` and `isize` types are not supported.

Comparison between [`musli-zerocopy`] and [`rkyv`].

Note that `musli-zerocopy` only supports the `primitives` benchmark.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/report/)
* [Sizes](#müsli-vs-rkyv-sizes)

<table>
<tr>
<th colspan="3">
<code>zerocopy-rkyv/dec/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_zerocopy-rkyv.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_zerocopy-rkyv.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primitives/musli_zerocopy` | **4.39ns** ± 0.00ns | 4.38ns &mdash; 4.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primitives/musli_zerocopy/report/) |
| `dec/primitives/rkyv` | **11.09ns** ± 0.01ns | 11.08ns &mdash; 11.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primitives/rkyv/report/) |

<table>
<tr>
<th colspan="3">
<code>zerocopy-rkyv/enc/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_zerocopy-rkyv.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_zerocopy-rkyv.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primitives/musli_zerocopy` | **18.67ns** ± 0.02ns | 18.63ns &mdash; 18.71ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primitives/musli_zerocopy/report/) |
| `enc/primitives/rkyv` | **17.99ns** ± 0.01ns | 17.96ns &mdash; 18.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primitives/rkyv/report/) |


<table>
<tr>
<th colspan="3">
<code>zerocopy-rkyv/dec/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_zerocopy-rkyv.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_zerocopy-rkyv.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primpacked/musli_zerocopy` | **2.63ns** ± 0.00ns | 2.63ns &mdash; 2.64ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primpacked/musli_zerocopy/report/) |
| `dec/primpacked/rkyv` | **8.77ns** ± 0.01ns | 8.76ns &mdash; 8.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primpacked/rkyv/report/) |

<table>
<tr>
<th colspan="3">
<code>zerocopy-rkyv/enc/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_zerocopy-rkyv.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_zerocopy-rkyv.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primpacked/musli_zerocopy` | **17.23ns** ± 0.02ns | 17.20ns &mdash; 17.27ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primpacked/musli_zerocopy/report/) |
| `enc/primpacked/rkyv` | **13.37ns** ± 0.01ns | 13.36ns &mdash; 13.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primpacked/rkyv/report/) |



### Müsli vs zerocopy

Compares [`musli-zerocopy`] with [`zerocopy`].

Note that `zerocopy` only supports packed primitives, so we're only comparing with that suite.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/report/)
* [Sizes](#müsli-vs-zerocopy-sizes)

<table>
<tr>
<th colspan="3">
<code>zerocopy-zerocopy/dec/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_zerocopy-zerocopy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_zerocopy-zerocopy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primpacked/musli_zerocopy` | **2.63ns** ± 0.00ns | 2.63ns &mdash; 2.64ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/musli_zerocopy/report/) |
| `dec/primpacked/zerocopy` | **5.26ns** ± 0.00ns | 5.26ns &mdash; 5.27ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/zerocopy/report/) |

<table>
<tr>
<th colspan="3">
<code>zerocopy-zerocopy/enc/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_zerocopy-zerocopy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_zerocopy-zerocopy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primpacked/musli_zerocopy` | **18.44ns** ± 0.02ns | 18.41ns &mdash; 18.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/musli_zerocopy/report/) |
| `enc/primpacked/zerocopy` | **7.94ns** ± 0.01ns | 7.93ns &mdash; 7.95ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/zerocopy/report/) |



### Bitcode derive

> **Missing features:**
> - `cstring` - `CString`'s are not supported.

Uses a custom derive-based framework which does not support everything Müsli and serde does.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/report/)
* [Sizes](#bitcode-derive-sizes)

<table>
<tr>
<th colspan="3">
<code>bitcode-derive/dec/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primitives/derive_bitcode` | **245.41ns** ± 0.26ns | 245.00ns &mdash; 246.01ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/derive_bitcode/report/) |
| `dec/primitives/musli_descriptive` | **987.51ns** ± 0.73ns | 986.43ns &mdash; 989.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **425.46ns** ± 0.44ns | 424.69ns &mdash; 426.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **93.95ns** ± 0.14ns | 93.71ns &mdash; 94.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_wire` | **904.91ns** ± 1.31ns | 902.86ns &mdash; 907.88ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bitcode-derive/enc/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primitives/derive_bitcode` | **1.28μs** ± 0.94ns | 1.27μs &mdash; 1.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/derive_bitcode/report/) |
| `enc/primitives/musli_descriptive` | **863.80ns** ± 0.61ns | 862.80ns &mdash; 865.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **292.02ns** ± 0.30ns | 291.53ns &mdash; 292.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **132.54ns** ± 0.12ns | 132.34ns &mdash; 132.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_wire` | **765.42ns** ± 1.00ns | 763.59ns &mdash; 767.50ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bitcode-derive/dec/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primpacked/derive_bitcode` | **244.16ns** ± 0.17ns | 243.90ns &mdash; 244.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/derive_bitcode/report/) |
| `dec/primpacked/musli_descriptive` | **972.94ns** ± 1.12ns | 971.37ns &mdash; 975.49ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **488.69ns** ± 0.90ns | 487.18ns &mdash; 490.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **487.37ns** ± 0.36ns | 486.73ns &mdash; 488.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_wire` | **916.50ns** ± 0.72ns | 915.35ns &mdash; 918.09ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bitcode-derive/enc/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primpacked/derive_bitcode` | **1.32μs** ± 1.71ns | 1.31μs &mdash; 1.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/derive_bitcode/report/) |
| `enc/primpacked/musli_descriptive` | **873.25ns** ± 0.96ns | 871.57ns &mdash; 875.29ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **231.99ns** ± 0.18ns | 231.70ns &mdash; 232.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **230.61ns** ± 0.16ns | 230.35ns &mdash; 230.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_wire` | **736.92ns** ± 0.87ns | 735.29ns &mdash; 738.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bitcode-derive/dec/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/medium_enum/derive_bitcode` | **3.27μs** ± 2.32ns | 3.27μs &mdash; 3.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/derive_bitcode/report/) |
| `dec/medium_enum/musli_descriptive` | **2.44μs** ± 1.92ns | 2.44μs &mdash; 2.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.36μs** ± 1.32ns | 1.36μs &mdash; 1.36μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **1.01μs** ± 1.11ns | 1.01μs &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_wire` | **2.07μs** ± 1.82ns | 2.07μs &mdash; 2.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bitcode-derive/enc/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/medium_enum/derive_bitcode` | **13.23μs** ± 15.20ns | 13.21μs &mdash; 13.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/derive_bitcode/report/) |
| `enc/medium_enum/musli_descriptive` | **1.58μs** ± 1.24ns | 1.57μs &mdash; 1.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **669.16ns** ± 0.47ns | 668.37ns &mdash; 670.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **499.98ns** ± 0.42ns | 499.23ns &mdash; 500.86ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_wire` | **1.34μs** ± 1.36ns | 1.34μs &mdash; 1.34μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bitcode-derive/dec/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/large/derive_bitcode` | **32.86μs** ± 39.23ns | 32.79μs &mdash; 32.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/derive_bitcode/report/) |
| `dec/large/musli_descriptive` | **275.57μs** ± 180.35ns | 275.27μs &mdash; 275.97μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **92.76μs** ± 64.70ns | 92.66μs &mdash; 92.91μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **50.48μs** ± 54.71ns | 50.38μs &mdash; 50.60μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_wire` | **258.42μs** ± 344.75ns | 257.97μs &mdash; 259.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bitcode-derive/enc/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/large/derive_bitcode` | **84.21μs** ± 151.77ns | 83.99μs &mdash; 84.55μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/derive_bitcode/report/) |
| `enc/large/musli_descriptive` | **186.96μs** ± 166.31ns | 186.71μs &mdash; 187.34μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **63.90μs** ± 55.40ns | 63.82μs &mdash; 64.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **29.68μs** ± 21.35ns | 29.64μs &mdash; 29.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_wire` | **159.80μs** ± 126.97ns | 159.60μs &mdash; 160.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bitcode-derive/dec/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/allocated/derive_bitcode` | **3.95μs** ± 5.32ns | 3.94μs &mdash; 3.96μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/derive_bitcode/report/) |
| `dec/allocated/musli_descriptive` | **3.90μs** ± 2.44ns | 3.90μs &mdash; 3.91μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **3.27μs** ± 3.81ns | 3.26μs &mdash; 3.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.98μs** ± 1.97ns | 2.98μs &mdash; 2.99μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_wire` | **4.08μs** ± 3.36ns | 4.07μs &mdash; 4.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bitcode-derive/enc/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/allocated/derive_bitcode` | **7.02μs** ± 11.28ns | 7.00μs &mdash; 7.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/derive_bitcode/report/) |
| `enc/allocated/musli_descriptive` | **808.57ns** ± 0.52ns | 807.71ns &mdash; 809.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **374.40ns** ± 0.39ns | 373.75ns &mdash; 375.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **304.21ns** ± 0.20ns | 303.88ns &mdash; 304.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_wire` | **734.79ns** ± 0.71ns | 733.57ns &mdash; 736.33ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_wire/report/) |



### BSON

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `u64` - Format is limited to the bounds of signed 64-bit integers.
> - `empty` - Empty variants are not supported.
> - `newtype` - Newtype variants are not supported.
> - `number-key` - Maps with numerical keys like `HashMap<u32, T>` are not supported.

Specific comparison to BSON, because the format is limited in capabilities.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/report/)
* [Sizes](#bson-sizes)

<table>
<tr>
<th colspan="3">
<code>bson/dec/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primitives/bson`[^bson] | **2.27μs** ± 4.40ns | 2.26μs &mdash; 2.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/bson/report/) |
| `dec/primitives/musli_descriptive` | **707.83ns** ± 0.67ns | 706.66ns &mdash; 709.28ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **378.56ns** ± 0.47ns | 377.98ns &mdash; 379.60ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **82.90ns** ± 0.09ns | 82.79ns &mdash; 83.09ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_wire` | **643.86ns** ± 0.71ns | 642.88ns &mdash; 645.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bson/enc/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primitives/bson`[^bson] | **1.32μs** ± 1.22ns | 1.31μs &mdash; 1.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/bson/report/) |
| `enc/primitives/musli_descriptive` | **532.31ns** ± 0.38ns | 531.65ns &mdash; 533.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **257.45ns** ± 0.27ns | 257.05ns &mdash; 258.07ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **120.59ns** ± 0.08ns | 120.46ns &mdash; 120.77ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_wire` | **416.91ns** ± 0.47ns | 416.14ns &mdash; 417.97ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bson/dec/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primpacked/bson`[^bson] | **3.04μs** ± 4.75ns | 3.03μs &mdash; 3.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/bson/report/) |
| `dec/primpacked/musli_descriptive` | **735.76ns** ± 0.55ns | 734.82ns &mdash; 736.95ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **403.59ns** ± 0.26ns | 403.16ns &mdash; 404.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **404.12ns** ± 0.49ns | 403.29ns &mdash; 405.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_wire` | **696.79ns** ± 0.61ns | 695.97ns &mdash; 698.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bson/enc/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primpacked/bson`[^bson] | **2.24μs** ± 2.03ns | 2.24μs &mdash; 2.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/bson/report/) |
| `enc/primpacked/musli_descriptive` | **546.46ns** ± 0.38ns | 545.84ns &mdash; 547.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **213.73ns** ± 0.13ns | 213.51ns &mdash; 214.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **213.82ns** ± 0.17ns | 213.53ns &mdash; 214.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_wire` | **462.48ns** ± 0.51ns | 461.50ns &mdash; 463.49ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bson/dec/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/medium_enum/bson`[^bson] | **7.32μs** ± 10.08ns | 7.30μs &mdash; 7.34μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/bson/report/) |
| `dec/medium_enum/musli_descriptive` | **1.56μs** ± 2.12ns | 1.55μs &mdash; 1.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **878.37ns** ± 1.33ns | 876.30ns &mdash; 881.38ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **583.09ns** ± 0.36ns | 582.53ns &mdash; 583.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_wire` | **1.32μs** ± 1.07ns | 1.32μs &mdash; 1.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bson/enc/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/medium_enum/bson`[^bson] | **4.79μs** ± 4.23ns | 4.78μs &mdash; 4.79μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/bson/report/) |
| `enc/medium_enum/musli_descriptive` | **1.15μs** ± 0.80ns | 1.15μs &mdash; 1.15μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **621.99ns** ± 0.37ns | 621.38ns &mdash; 622.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **401.14ns** ± 0.26ns | 400.71ns &mdash; 401.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_wire` | **840.45ns** ± 0.60ns | 839.53ns &mdash; 841.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bson/dec/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/large/bson`[^bson] | **1.56ms** ± 2.30μs | 1.56ms &mdash; 1.57ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/bson/report/) |
| `dec/large/musli_descriptive` | **359.97μs** ± 227.21ns | 359.59μs &mdash; 360.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **148.37μs** ± 119.84ns | 148.18μs &mdash; 148.64μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **88.68μs** ± 69.49ns | 88.57μs &mdash; 88.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_wire` | **339.61μs** ± 460.64ns | 338.93μs &mdash; 340.67μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bson/enc/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/large/bson`[^bson] | **895.49μs** ± 957.18ns | 893.98μs &mdash; 897.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/bson/report/) |
| `enc/large/musli_descriptive` | **232.10μs** ± 274.39ns | 231.71μs &mdash; 232.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **87.55μs** ± 73.82ns | 87.45μs &mdash; 87.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **45.32μs** ± 35.26ns | 45.26μs &mdash; 45.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_wire` | **188.99μs** ± 208.58ns | 188.64μs &mdash; 189.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bson/dec/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/allocated/bson`[^bson] | **8.28μs** ± 7.23ns | 8.26μs &mdash; 8.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/bson/report/) |
| `dec/allocated/musli_descriptive` | **3.06μs** ± 3.34ns | 3.05μs &mdash; 3.07μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **2.77μs** ± 4.56ns | 2.76μs &mdash; 2.78μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.55μs** ± 2.53ns | 2.54μs &mdash; 2.55μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_wire` | **2.95μs** ± 3.64ns | 2.94μs &mdash; 2.96μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bson/enc/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/allocated/bson`[^bson] | **2.29μs** ± 1.49ns | 2.29μs &mdash; 2.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/bson/report/) |
| `enc/allocated/musli_descriptive` | **539.12ns** ± 0.41ns | 538.43ns &mdash; 540.01ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **324.64ns** ± 0.33ns | 324.16ns &mdash; 325.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **259.29ns** ± 0.24ns | 258.88ns &mdash; 259.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_wire` | **433.67ns** ± 0.37ns | 433.00ns &mdash; 434.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_wire/report/) |



# Size comparisons

This is not yet an area which has received much focus, but because people are bound to ask the following section performs a raw size comparison between different formats.
Each test suite serializes a collection of values, which have all been randomly populated.
- A small object containing one of each primitive type and a string and a byte array. (`primitives`)
- Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries. (`primpacked`)
- A moderately sized enum with every kind of supported variant. (`medium_enum`)
- A really big and complex struct. (`large`)
- A sparse struct which contains fairly plain allocated data like strings and vectors. (`allocated`)

> **Note** so far these are all synthetic examples. Real world data is
> rarely *this* random. But hopefully it should give an idea of the extreme
> ranges.

#### Full features sizes

These frameworks provide a fair comparison against Müsli on various areas since
they support the same set of features in what types of data they can represent.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `musli_descriptive` | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 22219, max: 66506, stddev: 11024.160833823134">49227.10 ± 11024.16</a> | <a title="samples: 100, min: 361, max: 948, stddev: 114.7006761096028">647.93 ± 114.70</a> | <a title="samples: 4000, min: 4, max: 191, stddev: 64.94385785767209">53.81 ± 64.94</a> |
| `musli_storage` | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 16992, max: 46342, stddev: 7269.85903577229">34773.00 ± 7269.86</a> | <a title="samples: 100, min: 324, max: 896, stddev: 112.02235669722363">604.54 ± 112.02</a> | <a title="samples: 4000, min: 2, max: 149, stddev: 49.58829789123677">40.90 ± 49.59</a> |
| `musli_storage_packed` | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 15892, max: 42502, stddev: 6580.6585909010655">31855.10 ± 6580.66</a> | <a title="samples: 100, min: 312, max: 884, stddev: 112.02235669722363">592.54 ± 112.02</a> | <a title="samples: 4000, min: 2, max: 149, stddev: 45.86285847774888">38.77 ± 45.86</a> |
| `musli_wire` | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 20849, max: 61965, stddev: 10213.501348705056">45860.00 ± 10213.50</a> | <a title="samples: 100, min: 348, max: 936, stddev: 114.74952679640991">634.69 ± 114.75</a> | <a title="samples: 4000, min: 3, max: 179, stddev: 59.49902234438409">49.38 ± 59.50</a> |
| `postcard` | <a title="samples: 500, min: 105, max: 114, stddev: 1.4079360780944647">110.85 ± 1.41</a> | <a title="samples: 500, min: 107, max: 114, stddev: 1.3359101766211645">110.81 ± 1.34</a> | <a title="samples: 10, min: 16823, max: 45980, stddev: 7216.315362427005">34448.30 ± 7216.32</a> | <a title="samples: 100, min: 323, max: 901, stddev: 113.00814306942662">605.86 ± 113.01</a> | <a title="samples: 4000, min: 1, max: 146, stddev: 48.10210297897552">39.62 ± 48.10</a> |
| `serde_bincode` | <a title="samples: 500, min: 93, max: 95, stddev: 0.20591260281973842">94.96 ± 0.21</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 16585, max: 43238, stddev: 6612.325921338118">32444.10 ± 6612.33</a> | <a title="samples: 100, min: 416, max: 1009, stddev: 117.76399237457943">710.89 ± 117.76</a> | <a title="samples: 4000, min: 4, max: 163, stddev: 47.269325396471714">42.39 ± 47.27</a> |
| `serde_bitcode` | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 105, max: 105, stddev: 0">105.00 ± 0.00</a> | <a title="samples: 10, min: 15272, max: 39664, stddev: 6055.103051146199">29918.20 ± 6055.10</a> | <a title="samples: 100, min: 320, max: 892, stddev: 112.02235669722363">600.54 ± 112.02</a> | <a title="samples: 4000, min: 1, max: 147, stddev: 46.904018495220846">38.76 ± 46.90</a> |
| `serde_rmp` | <a title="samples: 500, min: 111, max: 115, stddev: 0.7291090453423233">113.82 ± 0.73</a> | <a title="samples: 500, min: 116, max: 123, stddev: 1.4824304368165206">119.88 ± 1.48</a> | <a title="samples: 10, min: 18609, max: 52430, stddev: 8350.44791852509">38929.60 ± 8350.45</a> | <a title="samples: 100, min: 328, max: 910, stddev: 113.80334749030892">612.41 ± 113.80</a> | <a title="samples: 4000, min: 6, max: 173, stddev: 50.740470019009805">50.97 ± 50.74</a> |

#### Text-based formats sizes

These are text-based formats, which support the full feature set of this test suite.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `musli_json`[^incomplete] | <a title="samples: 500, min: 308, max: 322, stddev: 2.370359466410104">315.41 ± 2.37</a> | <a title="samples: 500, min: 326, max: 343, stddev: 2.9921657708088594">335.29 ± 2.99</a> | <a title="samples: 10, min: 37853, max: 127140, stddev: 22814.18436170796">93162.10 ± 22814.18</a> | <a title="samples: 100, min: 532, max: 1130, stddev: 118.61131649214592">832.66 ± 118.61</a> | <a title="samples: 4000, min: 8, max: 374, stddev: 115.65740907281904">86.17 ± 115.66</a> |
| `serde_json`[^incomplete] | <a title="samples: 500, min: 428, max: 442, stddev: 2.370359466410104">435.41 ± 2.37</a> | <a title="samples: 500, min: 443, max: 460, stddev: 2.9921657708088594">452.29 ± 2.99</a> | <a title="samples: 10, min: 42978, max: 136779, stddev: 24197.177306661204">102095.30 ± 24197.18</a> | <a title="samples: 100, min: 633, max: 1231, stddev: 118.61131649214592">933.66 ± 118.61</a> | <a title="samples: 4000, min: 7, max: 508, stddev: 155.60660260232385">107.17 ± 155.61</a> |

#### Fewer features sizes

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `map` - Maps are not supported.

This is a suite where support for 128-bit integers and maps are disabled.
Usually because the underlying framework lacks support for them.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `musli_descriptive` | <a title="samples: 500, min: 112, max: 120, stddev: 1.4613363746926964">116.36 ± 1.46</a> | <a title="samples: 500, min: 118, max: 126, stddev: 1.457772273024832">122.33 ± 1.46</a> | <a title="samples: 10, min: 17864, max: 47252, stddev: 9328.948976170896">30994.00 ± 9328.95</a> | <a title="samples: 100, min: 299, max: 737, stddev: 97.85942724132408">488.45 ± 97.86</a> | <a title="samples: 4000, min: 4, max: 181, stddev: 54.38790407572287">48.29 ± 54.39</a> |
| `musli_storage` | <a title="samples: 500, min: 78, max: 82, stddev: 0.7069257386741584">80.98 ± 0.71</a> | <a title="samples: 500, min: 81, max: 84, stddev: 0.7482539675805259">83.05 ± 0.75</a> | <a title="samples: 10, min: 11963, max: 30708, stddev: 5971.916925075231">20507.20 ± 5971.92</a> | <a title="samples: 100, min: 274, max: 704, stddev: 96.33370074901097">458.59 ± 96.33</a> | <a title="samples: 4000, min: 2, max: 148, stddev: 42.323093447904064">36.57 ± 42.32</a> |
| `musli_storage_packed` | <a title="samples: 500, min: 63, max: 67, stddev: 0.7069257386741584">65.98 ± 0.71</a> | <a title="samples: 500, min: 81, max: 84, stddev: 0.7482539675805259">83.05 ± 0.75</a> | <a title="samples: 10, min: 10728, max: 27628, stddev: 5341.413741697978">18267.20 ± 5341.41</a> | <a title="samples: 100, min: 264, max: 694, stddev: 96.33370074901097">448.59 ± 96.33</a> | <a title="samples: 4000, min: 2, max: 148, stddev: 39.82015650396179">34.69 ± 39.82</a> |
| `musli_wire` | <a title="samples: 500, min: 96, max: 106, stddev: 1.7524143345681649">101.86 ± 1.75</a> | <a title="samples: 500, min: 102, max: 111, stddev: 1.7655310815729104">106.83 ± 1.77</a> | <a title="samples: 10, min: 16463, max: 43841, stddev: 8662.557910917536">28513.80 ± 8662.56</a> | <a title="samples: 100, min: 288, max: 726, stddev: 98.035046794501">477.64 ± 98.04</a> | <a title="samples: 4000, min: 3, max: 173, stddev: 49.88740967418499">44.07 ± 49.89</a> |
| `serde_cbor`[^i128] | <a title="samples: 500, min: 210, max: 213, stddev: 0.5346961754117986">212.69 ± 0.53</a> | <a title="samples: 500, min: 218, max: 222, stddev: 0.847610759724064">221.17 ± 0.85</a> | <a title="samples: 10, min: 20033, max: 47027, stddev: 9429.151033364564">34759.30 ± 9429.15</a> | <a title="samples: 100, min: 380, max: 815, stddev: 97.29138656633484">566.69 ± 97.29</a> | <a title="samples: 4000, min: 6, max: 251, stddev: 80.46084400152334">65.78 ± 80.46</a> |

#### Müsli vs rkyv sizes

> **Missing features:**
> - `cstring` - `CString`'s are not supported.
> - `string-key` - Maps with strings as keys like `HashMap<String, T>` are not supported.
> - `string-set` - String sets like `HashSet<String>` are not supported.
> - `tuple` - Tuples like `(u32, u32)` are not supported.
> - `usize` - `usize` and `isize` types are not supported.

Comparison between [`musli-zerocopy`] and [`rkyv`].

Note that `musli-zerocopy` only supports the `primitives` benchmark.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `musli_zerocopy` | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 500, min: 80, max: 80, stddev: 0">80.00 ± 0.00</a> | - | - | - |
| `rkyv`[^incomplete] | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 500, min: 80, max: 80, stddev: 0">80.00 ± 0.00</a> | <a title="samples: 10, min: 5700, max: 20372, stddev: 4086.943190209524">11350.40 ± 4086.94</a> | <a title="samples: 100, min: 312, max: 736, stddev: 78.54813556030469">519.48 ± 78.55</a> | <a title="samples: 4000, min: 128, max: 272, stddev: 39.772784866036105">148.83 ± 39.77</a> |

#### Müsli vs zerocopy sizes

Compares [`musli-zerocopy`] with [`zerocopy`].

Note that `zerocopy` only supports packed primitives, so we're only comparing with that suite.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `musli_zerocopy` | <a title="samples: 500, min: 112, max: 112, stddev: 0">112.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - |
| `zerocopy` | - | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - |

#### Bitcode derive sizes

> **Missing features:**
> - `cstring` - `CString`'s are not supported.

Uses a custom derive-based framework which does not support everything Müsli and serde does.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `derive_bitcode` | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 106, max: 106, stddev: 0">106.00 ± 0.00</a> | <a title="samples: 10, min: 15270, max: 39662, stddev: 6055.103051146199">29916.20 ± 6055.10</a> | <a title="samples: 100, min: 331, max: 869, stddev: 109.30754777232909">593.80 ± 109.31</a> | <a title="samples: 4000, min: 1, max: 147, stddev: 46.871152316083304">38.68 ± 46.87</a> |
| `musli_descriptive` | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 22219, max: 66506, stddev: 11024.160833823134">49227.10 ± 11024.16</a> | <a title="samples: 100, min: 363, max: 925, stddev: 111.50839026727988">639.17 ± 111.51</a> | <a title="samples: 4000, min: 4, max: 191, stddev: 64.9187642322117">53.74 ± 64.92</a> |
| `musli_storage` | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 16992, max: 46342, stddev: 7269.85903577229">34773.00 ± 7269.86</a> | <a title="samples: 100, min: 334, max: 872, stddev: 109.30754777232909">596.80 ± 109.31</a> | <a title="samples: 4000, min: 2, max: 149, stddev: 49.55776746383936">40.82 ± 49.56</a> |
| `musli_storage_packed` | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 15892, max: 42502, stddev: 6580.6585909010655">31855.10 ± 6580.66</a> | <a title="samples: 100, min: 323, max: 861, stddev: 109.30754777232909">585.80 ± 109.31</a> | <a title="samples: 4000, min: 2, max: 149, stddev: 45.8264846022475">38.70 ± 45.83</a> |
| `musli_wire` | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 20849, max: 61965, stddev: 10213.501348705056">45860.00 ± 10213.50</a> | <a title="samples: 100, min: 350, max: 912, stddev: 111.85884140290388">627.14 ± 111.86</a> | <a title="samples: 4000, min: 3, max: 179, stddev: 59.46716043119924">49.31 ± 59.47</a> |

#### BSON sizes

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `u64` - Format is limited to the bounds of signed 64-bit integers.
> - `empty` - Empty variants are not supported.
> - `newtype` - Newtype variants are not supported.
> - `number-key` - Maps with numerical keys like `HashMap<u32, T>` are not supported.

Specific comparison to BSON, because the format is limited in capabilities.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `bson` | <a title="samples: 500, min: 240, max: 241, stddev: 0.21794494717703397">240.95 ± 0.22</a> | <a title="samples: 500, min: 289, max: 289, stddev: 0">289.00 ± 0.00</a> | <a title="samples: 10, min: 44060, max: 121362, stddev: 27440.337983341244">88697.60 ± 27440.34</a> | <a title="samples: 100, min: 529, max: 1006, stddev: 104.64972575214902">759.57 ± 104.65</a> | <a title="samples: 2500, min: 22, max: 305, stddev: 114.417491890707">117.73 ± 114.42</a> |
| `musli_descriptive` | <a title="samples: 500, min: 111, max: 118, stddev: 1.3041027566875312">115.35 ± 1.30</a> | <a title="samples: 500, min: 118, max: 124, stddev: 1.283900307656329">121.34 ± 1.28</a> | <a title="samples: 10, min: 24558, max: 65542, stddev: 14363.988170421195">47190.80 ± 14363.99</a> | <a title="samples: 100, min: 367, max: 830, stddev: 103.1530605459673">591.31 ± 103.15</a> | <a title="samples: 2500, min: 4, max: 183, stddev: 59.959893795770185">58.34 ± 59.96</a> |
| `musli_storage` | <a title="samples: 500, min: 78, max: 81, stddev: 0.5250676146935734">80.45 ± 0.53</a> | <a title="samples: 500, min: 81, max: 83, stddev: 0.5157363667611599">82.50 ± 0.52</a> | <a title="samples: 10, min: 18111, max: 45593, stddev: 9632.819630824612">32636.40 ± 9632.82</a> | <a title="samples: 100, min: 345, max: 801, stddev: 101.37758085494052">564.69 ± 101.38</a> | <a title="samples: 2500, min: 2, max: 149, stddev: 44.569862326913054">41.91 ± 44.57</a> |
| `musli_storage_packed` | <a title="samples: 500, min: 63, max: 66, stddev: 0.5250676146935734">65.45 ± 0.53</a> | <a title="samples: 500, min: 81, max: 83, stddev: 0.5157363667611599">82.50 ± 0.52</a> | <a title="samples: 10, min: 16529, max: 41686, stddev: 8685.511915828567">29620.40 ± 8685.51</a> | <a title="samples: 100, min: 335, max: 791, stddev: 101.37758085494052">554.69 ± 101.38</a> | <a title="samples: 2500, min: 2, max: 149, stddev: 41.095866310859535">38.91 ± 41.10</a> |
| `musli_wire` | <a title="samples: 500, min: 95, max: 104, stddev: 1.5956490842287305">100.85 ± 1.60</a> | <a title="samples: 500, min: 101, max: 109, stddev: 1.5742934923323604">105.84 ± 1.57</a> | <a title="samples: 10, min: 22820, max: 60929, stddev: 13284.040493765442">43705.40 ± 13284.04</a> | <a title="samples: 100, min: 357, max: 820, stddev: 103.13010035872165">580.32 ± 103.13</a> | <a title="samples: 2500, min: 3, max: 179, stddev: 54.380138262420814">52.30 ± 54.38</a> |


[^bson]: BSON does not support serializing directly in-place [without patches](https://github.com/mongodb/bson-rust/pull/328). As a result it is expected to be much slower.
[^i128]: Lacks 128-bit support.
[^incomplete]: These formats do not support a wide range of Rust types. Exact level of support varies. But from a size perspective it makes size comparisons either unfair or simply an esoteric exercise since they can (or cannot) make stricter assumptions as a result.
[^musli_value]: `musli-value` is a heap-allocated, in-memory format. Deserialization is expected to be as fast as a dynamic in-memory structure can be traversed, but serialization requires a lot of allocations. It is only included for reference.
[`rkyv`]: https://docs.rs/rkyv
[`zerocopy`]: https://docs.rs/zerocopy
[`musli-zerocopy`]: https://docs.rs/musli-zerocopy
