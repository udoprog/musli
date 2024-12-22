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
- [**Miniserde**](#miniserde) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/report/), [Sizes](#miniserde-sizes))

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
| `dec/primitives/musli_descriptive` | **701.71ns** ± 0.84ns | 700.19ns &mdash; 703.48ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **641.52ns** ± 0.82ns | 640.03ns &mdash; 643.25ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **99.10ns** ± 0.12ns | 98.87ns &mdash; 99.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_value`[^musli_value] | **411.82ns** ± 0.83ns | 410.48ns &mdash; 413.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **698.98ns** ± 0.67ns | 697.77ns &mdash; 700.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_wire/report/) |
| `dec/primitives/postcard` | **269.15ns** ± 0.40ns | 268.44ns &mdash; 270.01ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/postcard/report/) |
| `dec/primitives/serde_bincode` | **135.59ns** ± 0.12ns | 135.37ns &mdash; 135.84ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bincode/report/) |
| `dec/primitives/serde_bitcode` | **1.25μs** ± 1.82ns | 1.24μs &mdash; 1.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bitcode/report/) |
| `dec/primitives/serde_rmp` | **319.69ns** ± 0.40ns | 318.96ns &mdash; 320.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_rmp/report/) |

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
| `enc/primitives/musli_descriptive` | **878.06ns** ± 1.36ns | 875.47ns &mdash; 880.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **715.83ns** ± 0.94ns | 714.18ns &mdash; 717.85ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **118.52ns** ± 0.13ns | 118.30ns &mdash; 118.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.13μs** ± 1.28ns | 1.12μs &mdash; 1.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **709.89ns** ± 2.12ns | 706.54ns &mdash; 714.68ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_wire/report/) |
| `enc/primitives/postcard` | **435.26ns** ± 0.60ns | 434.19ns &mdash; 436.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/postcard/report/) |
| `enc/primitives/serde_bincode` | **115.32ns** ± 0.11ns | 115.12ns &mdash; 115.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bincode/report/) |
| `enc/primitives/serde_bitcode` | **3.82μs** ± 5.66ns | 3.81μs &mdash; 3.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bitcode/report/) |
| `enc/primitives/serde_rmp` | **266.27ns** ± 0.22ns | 265.86ns &mdash; 266.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_rmp/report/) |


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
| `dec/primpacked/musli_descriptive` | **719.90ns** ± 0.58ns | 718.85ns &mdash; 721.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **653.76ns** ± 0.98ns | 652.00ns &mdash; 655.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **94.17ns** ± 0.11ns | 93.97ns &mdash; 94.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **459.76ns** ± 0.55ns | 458.78ns &mdash; 460.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **732.40ns** ± 2.03ns | 729.52ns &mdash; 737.06ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/postcard` | **264.91ns** ± 0.32ns | 264.34ns &mdash; 265.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/postcard/report/) |
| `dec/primpacked/serde_bincode` | **104.46ns** ± 0.10ns | 104.28ns &mdash; 104.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_bincode/report/) |
| `dec/primpacked/serde_bitcode` | **1.53μs** ± 2.28ns | 1.52μs &mdash; 1.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_bitcode/report/) |
| `dec/primpacked/serde_rmp` | **406.48ns** ± 0.67ns | 405.33ns &mdash; 407.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_rmp/report/) |

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
| `enc/primpacked/musli_descriptive` | **780.55ns** ± 0.98ns | 778.73ns &mdash; 782.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **670.65ns** ± 1.22ns | 668.37ns &mdash; 673.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **113.61ns** ± 0.14ns | 113.35ns &mdash; 113.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **1.46μs** ± 1.27ns | 1.45μs &mdash; 1.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **667.46ns** ± 1.33ns | 664.95ns &mdash; 670.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/postcard` | **426.58ns** ± 0.39ns | 425.86ns &mdash; 427.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/postcard/report/) |
| `enc/primpacked/serde_bincode` | **127.27ns** ± 0.14ns | 127.01ns &mdash; 127.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_bincode/report/) |
| `enc/primpacked/serde_bitcode` | **4.59μs** ± 5.02ns | 4.58μs &mdash; 4.60μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_bitcode/report/) |
| `enc/primpacked/serde_rmp` | **326.22ns** ± 0.41ns | 325.50ns &mdash; 327.08ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_rmp/report/) |


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
| `dec/medium_enum/musli_descriptive` | **2.01μs** ± 2.87ns | 2.01μs &mdash; 2.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.45μs** ± 1.61ns | 1.45μs &mdash; 1.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **841.05ns** ± 0.96ns | 839.44ns &mdash; 843.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **1.01μs** ± 1.11ns | 1.01μs &mdash; 1.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **1.69μs** ± 2.48ns | 1.69μs &mdash; 1.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/postcard` | **1.19μs** ± 1.12ns | 1.19μs &mdash; 1.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/postcard/report/) |
| `dec/medium_enum/serde_bincode` | **932.64ns** ± 1.08ns | 930.70ns &mdash; 934.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_bincode/report/) |
| `dec/medium_enum/serde_bitcode` | **9.33μs** ± 17.18ns | 9.30μs &mdash; 9.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_bitcode/report/) |
| `dec/medium_enum/serde_rmp` | **2.40μs** ± 3.13ns | 2.39μs &mdash; 2.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_rmp/report/) |

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
| `enc/medium_enum/musli_descriptive` | **1.56μs** ± 1.45ns | 1.56μs &mdash; 1.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **1.23μs** ± 1.76ns | 1.22μs &mdash; 1.23μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **432.06ns** ± 0.40ns | 431.34ns &mdash; 432.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **3.11μs** ± 3.29ns | 3.11μs &mdash; 3.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **1.26μs** ± 1.45ns | 1.25μs &mdash; 1.26μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/postcard` | **892.40ns** ± 1.37ns | 890.05ns &mdash; 895.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/postcard/report/) |
| `enc/medium_enum/serde_bincode` | **347.51ns** ± 0.42ns | 346.81ns &mdash; 348.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_bincode/report/) |
| `enc/medium_enum/serde_bitcode` | **13.06μs** ± 15.74ns | 13.03μs &mdash; 13.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_bitcode/report/) |
| `enc/medium_enum/serde_rmp` | **729.49ns** ± 2.88ns | 725.69ns &mdash; 735.94ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_rmp/report/) |


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
| `dec/large/musli_descriptive` | **285.96μs** ± 445.66ns | 285.16μs &mdash; 286.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **189.29μs** ± 212.30ns | 188.92μs &mdash; 189.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **101.46μs** ± 138.60ns | 101.23μs &mdash; 101.77μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_value`[^musli_value] | **127.60μs** ± 250.02ns | 127.17μs &mdash; 128.15μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **222.98μs** ± 247.29ns | 222.57μs &mdash; 223.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_wire/report/) |
| `dec/large/postcard` | **89.69μs** ± 106.72ns | 89.52μs &mdash; 89.93μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/postcard/report/) |
| `dec/large/serde_bincode` | **69.86μs** ± 103.83ns | 69.67μs &mdash; 70.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bincode/report/) |
| `dec/large/serde_bitcode` | **98.58μs** ± 152.42ns | 98.31μs &mdash; 98.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bitcode/report/) |
| `dec/large/serde_rmp` | **216.72μs** ± 300.64ns | 216.20μs &mdash; 217.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_rmp/report/) |

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
| `enc/large/musli_descriptive` | **170.13μs** ± 176.15ns | 169.81μs &mdash; 170.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **136.29μs** ± 115.14ns | 136.08μs &mdash; 136.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **46.38μs** ± 49.61ns | 46.29μs &mdash; 46.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_value`[^musli_value] | **718.17μs** ± 2.48μs | 714.86μs &mdash; 723.77μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **147.23μs** ± 169.51ns | 146.92μs &mdash; 147.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_wire/report/) |
| `enc/large/postcard` | **113.04μs** ± 311.78ns | 112.51μs &mdash; 113.71μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/postcard/report/) |
| `enc/large/serde_bincode` | **42.62μs** ± 81.97ns | 42.48μs &mdash; 42.80μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bincode/report/) |
| `enc/large/serde_bitcode` | **107.46μs** ± 153.74ns | 107.20μs &mdash; 107.80μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bitcode/report/) |
| `enc/large/serde_rmp` | **155.32μs** ± 201.64ns | 154.96μs &mdash; 155.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_rmp/report/) |


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
| `dec/allocated/musli_descriptive` | **3.28μs** ± 5.70ns | 3.27μs &mdash; 3.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **3.07μs** ± 5.36ns | 3.06μs &mdash; 3.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.60μs** ± 4.30ns | 2.60μs &mdash; 2.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_value`[^musli_value] | **2.01μs** ± 1.62ns | 2.01μs &mdash; 2.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **3.24μs** ± 3.85ns | 3.23μs &mdash; 3.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_wire/report/) |
| `dec/allocated/postcard` | **3.45μs** ± 4.26ns | 3.44μs &mdash; 3.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/postcard/report/) |
| `dec/allocated/serde_bincode` | **3.27μs** ± 3.80ns | 3.26μs &mdash; 3.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bincode/report/) |
| `dec/allocated/serde_bitcode` | **6.07μs** ± 7.99ns | 6.05μs &mdash; 6.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bitcode/report/) |
| `dec/allocated/serde_rmp` | **4.14μs** ± 8.00ns | 4.12μs &mdash; 4.16μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_rmp/report/) |

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
| `enc/allocated/musli_descriptive` | **722.23ns** ± 0.83ns | 720.82ns &mdash; 724.07ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **675.74ns** ± 1.06ns | 673.86ns &mdash; 677.99ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **323.54ns** ± 0.82ns | 322.28ns &mdash; 325.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_value`[^musli_value] | **2.51μs** ± 2.87ns | 2.51μs &mdash; 2.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **632.58ns** ± 0.94ns | 630.83ns &mdash; 634.51ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_wire/report/) |
| `enc/allocated/postcard` | **1.21μs** ± 1.12ns | 1.20μs &mdash; 1.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/postcard/report/) |
| `enc/allocated/serde_bincode` | **389.09ns** ± 0.29ns | 388.58ns &mdash; 389.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bincode/report/) |
| `enc/allocated/serde_bitcode` | **8.34μs** ± 9.50ns | 8.32μs &mdash; 8.36μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bitcode/report/) |
| `enc/allocated/serde_rmp` | **781.84ns** ± 0.97ns | 780.04ns &mdash; 783.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_rmp/report/) |



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
| `dec/primitives/musli_json` | **3.65μs** ± 4.68ns | 3.64μs &mdash; 3.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **4.59μs** ± 5.32ns | 4.58μs &mdash; 4.60μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/serde_json/report/) |

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
| `enc/primitives/musli_json` | **1.32μs** ± 2.43ns | 1.32μs &mdash; 1.33μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **1.31μs** ± 1.49ns | 1.31μs &mdash; 1.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/serde_json/report/) |


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
| `dec/primpacked/musli_json` | **4.29μs** ± 4.43ns | 4.28μs &mdash; 4.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/musli_json/report/) |
| `dec/primpacked/serde_json` | **4.79μs** ± 6.25ns | 4.78μs &mdash; 4.80μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/serde_json/report/) |

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
| `enc/primpacked/musli_json` | **1.20μs** ± 3.06ns | 1.19μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/musli_json/report/) |
| `enc/primpacked/serde_json` | **1.38μs** ± 4.17ns | 1.37μs &mdash; 1.39μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/serde_json/report/) |


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
| `dec/medium_enum/musli_json` | **8.71μs** ± 21.55ns | 8.68μs &mdash; 8.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/musli_json/report/) |
| `dec/medium_enum/serde_json` | **8.37μs** ± 11.64ns | 8.35μs &mdash; 8.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/serde_json/report/) |

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
| `enc/medium_enum/musli_json` | **2.71μs** ± 2.86ns | 2.70μs &mdash; 2.71μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/musli_json/report/) |
| `enc/medium_enum/serde_json` | **2.53μs** ± 2.26ns | 2.53μs &mdash; 2.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/serde_json/report/) |


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
| `dec/large/musli_json` | **972.57μs** ± 1.51μs | 970.46μs &mdash; 976.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **767.34μs** ± 913.78ns | 765.89μs &mdash; 769.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/serde_json/report/) |

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
| `enc/large/musli_json` | **291.95μs** ± 310.08ns | 291.42μs &mdash; 292.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **304.61μs** ± 382.66ns | 303.96μs &mdash; 305.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/serde_json/report/) |


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
| `dec/allocated/musli_json` | **9.42μs** ± 12.37ns | 9.40μs &mdash; 9.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **7.60μs** ± 8.64ns | 7.58μs &mdash; 7.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/serde_json/report/) |

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
| `enc/allocated/musli_json` | **2.35μs** ± 2.63ns | 2.34μs &mdash; 2.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **2.43μs** ± 2.93ns | 2.42μs &mdash; 2.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/serde_json/report/) |



### Fewer features

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `map` - Maps like `MashMap<K, V>` are not supported.

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
| `dec/primitives/musli_descriptive` | **526.25ns** ± 0.51ns | 525.29ns &mdash; 527.29ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **493.96ns** ± 0.62ns | 492.79ns &mdash; 495.22ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **76.26ns** ± 0.09ns | 76.10ns &mdash; 76.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_value`[^musli_value] | **351.22ns** ± 0.33ns | 350.63ns &mdash; 351.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **540.71ns** ± 0.63ns | 539.59ns &mdash; 542.06ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_wire/report/) |
| `dec/primitives/serde_cbor` | **1.66μs** ± 1.67ns | 1.66μs &mdash; 1.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/serde_cbor/report/) |

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
| `enc/primitives/musli_descriptive` | **497.57ns** ± 1.01ns | 495.65ns &mdash; 499.62ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **395.52ns** ± 1.04ns | 394.06ns &mdash; 397.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **110.02ns** ± 0.29ns | 109.55ns &mdash; 110.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.01μs** ± 1.05ns | 1.01μs &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **376.46ns** ± 0.68ns | 375.15ns &mdash; 377.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_wire/report/) |
| `enc/primitives/serde_cbor` | **431.83ns** ± 0.49ns | 430.98ns &mdash; 432.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/serde_cbor/report/) |


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
| `dec/primpacked/musli_descriptive` | **580.12ns** ± 0.85ns | 578.58ns &mdash; 581.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **506.58ns** ± 0.56ns | 505.56ns &mdash; 507.76ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **72.42ns** ± 0.12ns | 72.20ns &mdash; 72.68ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **368.59ns** ± 0.33ns | 367.99ns &mdash; 369.27ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **564.17ns** ± 0.70ns | 562.88ns &mdash; 565.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/serde_cbor` | **1.78μs** ± 2.05ns | 1.77μs &mdash; 1.78μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/serde_cbor/report/) |

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
| `enc/primpacked/musli_descriptive` | **465.44ns** ± 0.55ns | 464.39ns &mdash; 466.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **476.02ns** ± 0.73ns | 474.72ns &mdash; 477.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **105.10ns** ± 0.11ns | 104.90ns &mdash; 105.33ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **1.19μs** ± 1.28ns | 1.19μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **341.86ns** ± 0.91ns | 340.19ns &mdash; 343.77ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/serde_cbor` | **484.72ns** ± 0.46ns | 483.89ns &mdash; 485.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/serde_cbor/report/) |


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
| `dec/medium_enum/musli_descriptive` | **1.92μs** ± 2.95ns | 1.92μs &mdash; 1.93μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.27μs** ± 1.67ns | 1.27μs &mdash; 1.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **816.74ns** ± 0.87ns | 815.13ns &mdash; 818.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **983.77ns** ± 1.11ns | 981.71ns &mdash; 986.05ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **1.51μs** ± 1.41ns | 1.50μs &mdash; 1.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/serde_cbor` | **4.65μs** ± 7.34ns | 4.63μs &mdash; 4.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/serde_cbor/report/) |

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
| `enc/medium_enum/musli_descriptive` | **1.15μs** ± 1.32ns | 1.15μs &mdash; 1.16μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **834.37ns** ± 1.11ns | 832.41ns &mdash; 836.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **417.59ns** ± 0.43ns | 416.80ns &mdash; 418.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **3.21μs** ± 3.87ns | 3.20μs &mdash; 3.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **899.24ns** ± 1.00ns | 897.40ns &mdash; 901.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/serde_cbor` | **1.03μs** ± 1.25ns | 1.03μs &mdash; 1.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/serde_cbor/report/) |


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
| `dec/large/musli_descriptive` | **311.95μs** ± 472.53ns | 311.14μs &mdash; 312.97μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **198.21μs** ± 202.05ns | 197.85μs &mdash; 198.64μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **99.10μs** ± 97.33ns | 98.93μs &mdash; 99.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_value`[^musli_value] | **137.45μs** ± 234.76ns | 137.04μs &mdash; 137.96μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **236.72μs** ± 275.37ns | 236.25μs &mdash; 237.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_wire/report/) |
| `dec/large/serde_cbor` | **576.62μs** ± 1.09μs | 574.94μs &mdash; 579.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/serde_cbor/report/) |

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
| `enc/large/musli_descriptive` | **187.26μs** ± 174.73ns | 186.94μs &mdash; 187.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **145.48μs** ± 144.53ns | 145.24μs &mdash; 145.80μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **52.06μs** ± 48.17ns | 51.97μs &mdash; 52.16μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_value`[^musli_value] | **768.83μs** ± 1.79μs | 766.02μs &mdash; 772.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **156.70μs** ± 165.94ns | 156.40μs &mdash; 157.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_wire/report/) |
| `enc/large/serde_cbor` | **174.03μs** ± 268.31ns | 173.57μs &mdash; 174.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/serde_cbor/report/) |


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
| `dec/allocated/musli_descriptive` | **2.30μs** ± 3.51ns | 2.29μs &mdash; 2.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **2.23μs** ± 4.02ns | 2.22μs &mdash; 2.24μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **1.90μs** ± 2.27ns | 1.89μs &mdash; 1.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_value`[^musli_value] | **1.45μs** ± 1.65ns | 1.45μs &mdash; 1.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **2.44μs** ± 2.02ns | 2.44μs &mdash; 2.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_wire/report/) |
| `dec/allocated/serde_cbor` | **4.87μs** ± 6.94ns | 4.86μs &mdash; 4.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/serde_cbor/report/) |

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
| `enc/allocated/musli_descriptive` | **556.99ns** ± 0.63ns | 555.82ns &mdash; 558.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **478.34ns** ± 0.85ns | 476.80ns &mdash; 480.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **251.57ns** ± 0.30ns | 251.02ns &mdash; 252.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_value`[^musli_value] | **1.96μs** ± 1.63ns | 1.96μs &mdash; 1.97μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **431.89ns** ± 0.65ns | 430.74ns &mdash; 433.28ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_wire/report/) |
| `enc/allocated/serde_cbor` | **647.19ns** ± 0.81ns | 645.70ns &mdash; 648.88ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/serde_cbor/report/) |



### Müsli vs rkyv

> **Missing features:**
> - `cstring` - `CString`'s are not supported.
> - `map` - Maps like `MashMap<K, V>` are not supported.
> - `set` - Sets like `HashSet<T>` are not supported.
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
| `dec/primitives/musli_zerocopy` | **3.99ns** ± 0.00ns | 3.98ns &mdash; 4.00ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primitives/musli_zerocopy/report/) |
| `dec/primitives/rkyv` | **14.72ns** ± 0.03ns | 14.67ns &mdash; 14.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primitives/rkyv/report/) |

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
| `enc/primitives/musli_zerocopy` | **19.97ns** ± 0.03ns | 19.91ns &mdash; 20.04ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primitives/musli_zerocopy/report/) |
| `enc/primitives/rkyv` | **118.95ns** ± 3.41ns | 112.12ns &mdash; 125.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primitives/rkyv/report/) |


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
| `dec/primpacked/musli_zerocopy` | **2.66ns** ± 0.00ns | 2.65ns &mdash; 2.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primpacked/musli_zerocopy/report/) |
| `dec/primpacked/rkyv` | **14.17ns** ± 0.02ns | 14.14ns &mdash; 14.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primpacked/rkyv/report/) |

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
| `enc/primpacked/musli_zerocopy` | **18.67ns** ± 0.03ns | 18.62ns &mdash; 18.74ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primpacked/musli_zerocopy/report/) |
| `enc/primpacked/rkyv` | **105.35ns** ± 2.84ns | 99.62ns &mdash; 110.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primpacked/rkyv/report/) |



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
| `dec/primpacked/musli_zerocopy` | **2.66ns** ± 0.00ns | 2.65ns &mdash; 2.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/musli_zerocopy/report/) |
| `dec/primpacked/zerocopy` | **6.65ns** ± 0.01ns | 6.64ns &mdash; 6.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/zerocopy/report/) |

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
| `enc/primpacked/musli_zerocopy` | **17.85ns** ± 0.02ns | 17.82ns &mdash; 17.88ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/musli_zerocopy/report/) |
| `enc/primpacked/zerocopy` | **8.41ns** ± 0.01ns | 8.40ns &mdash; 8.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/zerocopy/report/) |



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
| `dec/primitives/derive_bitcode` | **249.49ns** ± 0.27ns | 249.01ns &mdash; 250.07ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/derive_bitcode/report/) |
| `dec/primitives/musli_descriptive` | **692.13ns** ± 0.73ns | 690.81ns &mdash; 693.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **634.90ns** ± 1.79ns | 632.09ns &mdash; 638.95ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **98.71ns** ± 0.09ns | 98.55ns &mdash; 98.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_wire` | **693.98ns** ± 0.73ns | 692.70ns &mdash; 695.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/derive_bitcode` | **1.30μs** ± 1.74ns | 1.30μs &mdash; 1.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/derive_bitcode/report/) |
| `enc/primitives/musli_descriptive` | **872.07ns** ± 1.39ns | 869.45ns &mdash; 874.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **719.78ns** ± 1.08ns | 717.90ns &mdash; 722.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **118.30ns** ± 0.15ns | 118.02ns &mdash; 118.62ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_wire` | **705.00ns** ± 0.76ns | 703.65ns &mdash; 706.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_wire/report/) |


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
| `dec/primpacked/derive_bitcode` | **251.68ns** ± 0.31ns | 251.11ns &mdash; 252.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/derive_bitcode/report/) |
| `dec/primpacked/musli_descriptive` | **722.51ns** ± 0.87ns | 720.90ns &mdash; 724.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **655.49ns** ± 0.75ns | 654.09ns &mdash; 657.04ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **92.08ns** ± 0.13ns | 91.85ns &mdash; 92.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_wire` | **731.97ns** ± 0.90ns | 730.29ns &mdash; 733.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_wire/report/) |

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
| `enc/primpacked/derive_bitcode` | **1.33μs** ± 2.17ns | 1.33μs &mdash; 1.33μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/derive_bitcode/report/) |
| `enc/primpacked/musli_descriptive` | **782.29ns** ± 1.84ns | 778.96ns &mdash; 786.16ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **662.99ns** ± 1.14ns | 660.84ns &mdash; 665.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **113.50ns** ± 0.13ns | 113.26ns &mdash; 113.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_wire` | **659.44ns** ± 0.74ns | 658.06ns &mdash; 660.97ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_wire/report/) |


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
| `dec/medium_enum/derive_bitcode` | **3.28μs** ± 3.07ns | 3.27μs &mdash; 3.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/derive_bitcode/report/) |
| `dec/medium_enum/musli_descriptive` | **2.17μs** ± 2.57ns | 2.17μs &mdash; 2.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.61μs** ± 1.85ns | 1.61μs &mdash; 1.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **1.02μs** ± 1.21ns | 1.02μs &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_wire` | **1.84μs** ± 2.61ns | 1.83μs &mdash; 1.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_wire/report/) |

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
| `enc/medium_enum/derive_bitcode` | **13.42μs** ± 18.46ns | 13.39μs &mdash; 13.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/derive_bitcode/report/) |
| `enc/medium_enum/musli_descriptive` | **1.47μs** ± 1.78ns | 1.47μs &mdash; 1.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **1.18μs** ± 1.20ns | 1.18μs &mdash; 1.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **419.86ns** ± 0.46ns | 419.01ns &mdash; 420.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_wire` | **1.24μs** ± 1.87ns | 1.24μs &mdash; 1.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_wire/report/) |


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
| `dec/large/derive_bitcode` | **32.34μs** ± 42.49ns | 32.27μs &mdash; 32.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/derive_bitcode/report/) |
| `dec/large/musli_descriptive` | **290.18μs** ± 192.30ns | 289.83μs &mdash; 290.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **191.82μs** ± 145.99ns | 191.56μs &mdash; 192.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **100.29μs** ± 131.57ns | 100.05μs &mdash; 100.57μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_wire` | **227.02μs** ± 407.76ns | 226.34μs &mdash; 227.92μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_wire/report/) |

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
| `enc/large/derive_bitcode` | **86.34μs** ± 195.75ns | 86.00μs &mdash; 86.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/derive_bitcode/report/) |
| `enc/large/musli_descriptive` | **170.26μs** ± 201.01ns | 169.90μs &mdash; 170.69μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **136.22μs** ± 194.35ns | 135.87μs &mdash; 136.63μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **46.41μs** ± 59.43ns | 46.31μs &mdash; 46.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_wire` | **148.09μs** ± 223.08ns | 147.70μs &mdash; 148.57μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_wire/report/) |


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
| `dec/allocated/derive_bitcode` | **3.71μs** ± 5.65ns | 3.70μs &mdash; 3.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/derive_bitcode/report/) |
| `dec/allocated/musli_descriptive` | **3.77μs** ± 4.75ns | 3.76μs &mdash; 3.78μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **3.50μs** ± 4.80ns | 3.49μs &mdash; 3.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **3.01μs** ± 5.10ns | 3.00μs &mdash; 3.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_wire` | **3.68μs** ± 4.86ns | 3.67μs &mdash; 3.69μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_wire/report/) |

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
| `enc/allocated/derive_bitcode` | **7.09μs** ± 10.52ns | 7.07μs &mdash; 7.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/derive_bitcode/report/) |
| `enc/allocated/musli_descriptive` | **702.49ns** ± 0.71ns | 701.19ns &mdash; 703.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **648.97ns** ± 0.71ns | 647.66ns &mdash; 650.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **308.54ns** ± 0.38ns | 307.89ns &mdash; 309.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_wire` | **622.57ns** ± 0.88ns | 621.06ns &mdash; 624.48ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_wire/report/) |



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
| `dec/primitives/bson`[^bson] | **2.86μs** ± 3.68ns | 2.85μs &mdash; 2.87μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/bson/report/) |
| `dec/primitives/musli_descriptive` | **544.14ns** ± 0.99ns | 542.42ns &mdash; 546.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **471.84ns** ± 0.37ns | 471.13ns &mdash; 472.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **75.44ns** ± 0.08ns | 75.28ns &mdash; 75.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_wire` | **520.44ns** ± 0.72ns | 519.19ns &mdash; 521.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/bson`[^bson] | **1.37μs** ± 1.44ns | 1.36μs &mdash; 1.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/bson/report/) |
| `enc/primitives/musli_descriptive` | **478.08ns** ± 0.79ns | 476.73ns &mdash; 479.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **379.20ns** ± 0.66ns | 378.01ns &mdash; 380.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **99.21ns** ± 0.19ns | 98.91ns &mdash; 99.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_wire` | **348.76ns** ± 0.68ns | 347.45ns &mdash; 350.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_wire/report/) |


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
| `dec/primpacked/bson`[^bson] | **3.92μs** ± 6.27ns | 3.91μs &mdash; 3.93μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/bson/report/) |
| `dec/primpacked/musli_descriptive` | **563.11ns** ± 0.73ns | 561.89ns &mdash; 564.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **504.04ns** ± 0.80ns | 502.62ns &mdash; 505.75ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **67.45ns** ± 0.08ns | 67.30ns &mdash; 67.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_wire` | **552.39ns** ± 0.70ns | 551.17ns &mdash; 553.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_wire/report/) |

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
| `enc/primpacked/bson`[^bson] | **2.44μs** ± 4.62ns | 2.43μs &mdash; 2.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/bson/report/) |
| `enc/primpacked/musli_descriptive` | **440.52ns** ± 0.80ns | 439.12ns &mdash; 442.25ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **339.61ns** ± 1.01ns | 337.75ns &mdash; 341.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **100.19ns** ± 0.14ns | 99.95ns &mdash; 100.49ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_wire` | **316.58ns** ± 0.45ns | 315.74ns &mdash; 317.49ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_wire/report/) |


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
| `dec/medium_enum/bson`[^bson] | **7.96μs** ± 10.51ns | 7.94μs &mdash; 7.98μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/bson/report/) |
| `dec/medium_enum/musli_descriptive` | **1.59μs** ± 1.87ns | 1.58μs &mdash; 1.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **980.05ns** ± 1.53ns | 977.40ns &mdash; 983.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **582.70ns** ± 0.58ns | 581.64ns &mdash; 583.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_wire` | **1.18μs** ± 1.61ns | 1.18μs &mdash; 1.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_wire/report/) |

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
| `enc/medium_enum/bson`[^bson] | **5.21μs** ± 6.06ns | 5.20μs &mdash; 5.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/bson/report/) |
| `enc/medium_enum/musli_descriptive` | **909.69ns** ± 0.84ns | 908.25ns &mdash; 911.50ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **700.89ns** ± 1.10ns | 698.89ns &mdash; 703.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **329.61ns** ± 0.40ns | 328.87ns &mdash; 330.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_wire` | **727.39ns** ± 2.77ns | 723.48ns &mdash; 733.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_wire/report/) |


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
| `dec/large/bson`[^bson] | **1.77ms** ± 1.38μs | 1.77ms &mdash; 1.77ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/bson/report/) |
| `dec/large/musli_descriptive` | **383.38μs** ± 520.39ns | 382.47μs &mdash; 384.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **251.02μs** ± 523.22ns | 250.22μs &mdash; 252.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **139.94μs** ± 150.79ns | 139.67μs &mdash; 140.26μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_wire` | **295.70μs** ± 454.38ns | 294.92μs &mdash; 296.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_wire/report/) |

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
| `enc/large/bson`[^bson] | **985.01μs** ± 1.03μs | 983.11μs &mdash; 987.16μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/bson/report/) |
| `enc/large/musli_descriptive` | **199.90μs** ± 247.90ns | 199.48μs &mdash; 200.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **157.91μs** ± 251.54ns | 157.45μs &mdash; 158.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **59.58μs** ± 74.25ns | 59.45μs &mdash; 59.74μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_wire` | **164.41μs** ± 200.72ns | 164.07μs &mdash; 164.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_wire/report/) |


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
| `dec/allocated/bson`[^bson] | **7.52μs** ± 9.66ns | 7.51μs &mdash; 7.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/bson/report/) |
| `dec/allocated/musli_descriptive` | **3.01μs** ± 3.09ns | 3.00μs &mdash; 3.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **2.94μs** ± 3.76ns | 2.93μs &mdash; 2.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.62μs** ± 2.10ns | 2.61μs &mdash; 2.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_wire` | **3.04μs** ± 2.66ns | 3.03μs &mdash; 3.04μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_wire/report/) |

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
| `enc/allocated/bson`[^bson] | **2.52μs** ± 4.52ns | 2.51μs &mdash; 2.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/bson/report/) |
| `enc/allocated/musli_descriptive` | **462.95ns** ± 0.55ns | 461.96ns &mdash; 464.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **431.39ns** ± 0.71ns | 430.14ns &mdash; 432.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **257.82ns** ± 0.28ns | 257.30ns &mdash; 258.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_wire` | **358.54ns** ± 0.44ns | 357.77ns &mdash; 359.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_wire/report/) |



### Miniserde

> **Missing features:**
> - `cstring` - `CString`'s are not supported.
> - `btree`
> - `map` - Maps like `MashMap<K, V>` are not supported.
> - `set` - Sets like `HashSet<T>` are not supported.
> - `nonunit-variant` - Only empty unit variants are supported.
> - `128` - 128-bit integers are not supported.
> - `char` - Character types like `char` are not supported.
> - `tuple` - Tuples like `(u32, u32)` are not supported.
> - `binary-equality` - Values are not preserved exactly when serialized and deserialized. Such as floating point values, even when they are exact.

An experimental framework which only supports JSON and a limited number of Rust types.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/report/)
* [Sizes](#miniserde-sizes)

<table>
<tr>
<th colspan="3">
<code>miniserde/dec/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primitives/miniserde` | **2.09μs** ± 2.15ns | 2.08μs &mdash; 2.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/miniserde/report/) |
| `dec/primitives/musli_json` | **2.57μs** ± 5.40ns | 2.56μs &mdash; 2.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **2.31μs** ± 3.05ns | 2.31μs &mdash; 2.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>miniserde/enc/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primitives/miniserde` | **2.50μs** ± 2.71ns | 2.50μs &mdash; 2.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/miniserde/report/) |
| `enc/primitives/musli_json` | **959.86ns** ± 1.16ns | 957.76ns &mdash; 962.27ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **961.32ns** ± 1.04ns | 959.41ns &mdash; 963.49ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>miniserde/dec/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primpacked/miniserde` | **2.85μs** ± 3.93ns | 2.84μs &mdash; 2.86μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/miniserde/report/) |
| `dec/primpacked/musli_json` | **3.34μs** ± 3.28ns | 3.34μs &mdash; 3.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/musli_json/report/) |
| `dec/primpacked/serde_json` | **2.83μs** ± 3.19ns | 2.83μs &mdash; 2.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>miniserde/enc/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primpacked/miniserde` | **2.99μs** ± 3.02ns | 2.98μs &mdash; 3.00μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/miniserde/report/) |
| `enc/primpacked/musli_json` | **909.42ns** ± 0.91ns | 907.80ns &mdash; 911.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/musli_json/report/) |
| `enc/primpacked/serde_json` | **1.14μs** ± 1.36ns | 1.14μs &mdash; 1.15μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>miniserde/dec/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/medium_enum/miniserde` | **68.52ns** ± 0.08ns | 68.38ns &mdash; 68.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/miniserde/report/) |
| `dec/medium_enum/musli_json` | **64.70ns** ± 0.06ns | 64.58ns &mdash; 64.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/musli_json/report/) |
| `dec/medium_enum/serde_json` | **73.56ns** ± 0.08ns | 73.42ns &mdash; 73.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>miniserde/enc/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/medium_enum/miniserde` | **93.42ns** ± 0.10ns | 93.23ns &mdash; 93.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/miniserde/report/) |
| `enc/medium_enum/musli_json` | **23.99ns** ± 0.02ns | 23.95ns &mdash; 24.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/musli_json/report/) |
| `enc/medium_enum/serde_json` | **28.99ns** ± 0.03ns | 28.93ns &mdash; 29.06ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>miniserde/dec/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/large/miniserde` | **187.77μs** ± 335.54ns | 187.21μs &mdash; 188.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/miniserde/report/) |
| `dec/large/musli_json` | **246.52μs** ± 183.99ns | 246.18μs &mdash; 246.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **221.72μs** ± 246.50ns | 221.29μs &mdash; 222.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>miniserde/enc/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/large/miniserde` | **152.94μs** ± 338.36ns | 152.31μs &mdash; 153.63μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/miniserde/report/) |
| `enc/large/musli_json` | **93.05μs** ± 152.58ns | 92.78μs &mdash; 93.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **93.72μs** ± 186.08ns | 93.38μs &mdash; 94.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>miniserde/dec/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/allocated/miniserde` | **586.88ns** ± 0.74ns | 585.54ns &mdash; 588.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/miniserde/report/) |
| `dec/allocated/musli_json` | **570.77ns** ± 0.57ns | 569.70ns &mdash; 571.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **404.01ns** ± 0.40ns | 403.25ns &mdash; 404.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>miniserde/enc/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/allocated/miniserde` | **664.95ns** ± 1.03ns | 663.13ns &mdash; 667.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/miniserde/report/) |
| `enc/allocated/musli_json` | **132.54ns** ± 0.16ns | 132.26ns &mdash; 132.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **158.87ns** ± 0.22ns | 158.48ns &mdash; 159.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/serde_json/report/) |



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
| `musli_storage` | <a title="samples: 500, min: 122, max: 131, stddev: 1.3556681009745792">127.86 ± 1.36</a> | <a title="samples: 500, min: 127, max: 134, stddev: 1.3827783625729677">130.88 ± 1.38</a> | <a title="samples: 10, min: 18742, max: 54036, stddev: 8751.096357028644">40196.50 ± 8751.10</a> | <a title="samples: 100, min: 336, max: 914, stddev: 113.00814306942662">618.86 ± 113.01</a> | <a title="samples: 4000, min: 2, max: 151, stddev: 52.96969946995643">43.38 ± 52.97</a> |
| `musli_storage_packed` | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 98, max: 100, stddev: 0.7276785004382086">99.02 ± 0.73</a> | <a title="samples: 10, min: 16303, max: 44921, stddev: 7066.113501069736">33476.30 ± 7066.11</a> | <a title="samples: 100, min: 312, max: 884, stddev: 112.02235669722363">592.54 ± 112.02</a> | <a title="samples: 4000, min: 2, max: 149, stddev: 45.86285847774888">38.77 ± 45.86</a> |
| `musli_wire` | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 20849, max: 61965, stddev: 10213.501348705056">45860.00 ± 10213.50</a> | <a title="samples: 100, min: 348, max: 936, stddev: 114.74952679640991">634.69 ± 114.75</a> | <a title="samples: 4000, min: 3, max: 179, stddev: 59.49902234438409">49.38 ± 59.50</a> |
| `postcard` | <a title="samples: 500, min: 105, max: 114, stddev: 1.4079360780944647">110.85 ± 1.41</a> | <a title="samples: 500, min: 107, max: 114, stddev: 1.3359101766211645">110.81 ± 1.34</a> | <a title="samples: 10, min: 16823, max: 45980, stddev: 7216.315362427005">34448.30 ± 7216.32</a> | <a title="samples: 100, min: 323, max: 901, stddev: 113.00814306942662">605.86 ± 113.01</a> | <a title="samples: 4000, min: 1, max: 146, stddev: 48.10210297897552">39.62 ± 48.10</a> |
| `serde_bincode` | <a title="samples: 500, min: 93, max: 95, stddev: 0.20591260281973842">94.96 ± 0.21</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 16585, max: 43238, stddev: 6612.325921338118">32444.10 ± 6612.33</a> | <a title="samples: 100, min: 416, max: 1009, stddev: 117.76399237457943">710.89 ± 117.76</a> | <a title="samples: 4000, min: 4, max: 163, stddev: 47.269325396471714">42.39 ± 47.27</a> |
| `serde_bitcode` | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 105, max: 105, stddev: 0">105.00 ± 0.00</a> | <a title="samples: 10, min: 15272, max: 39664, stddev: 6055.103051146199">29918.20 ± 6055.10</a> | <a title="samples: 100, min: 320, max: 892, stddev: 112.02235669722363">600.54 ± 112.02</a> | <a title="samples: 4000, min: 1, max: 147, stddev: 46.904018495220846">38.76 ± 46.90</a> |
| `serde_rmp` | <a title="samples: 500, min: 111, max: 115, stddev: 0.7291090453423233">113.82 ± 0.73</a> | <a title="samples: 500, min: 116, max: 123, stddev: 1.4824304368165206">119.88 ± 1.48</a> | <a title="samples: 10, min: 18609, max: 52430, stddev: 8350.44791852509">38929.60 ± 8350.45</a> | <a title="samples: 100, min: 328, max: 910, stddev: 113.80334749030892">612.41 ± 113.80</a> | <a title="samples: 4000, min: 6, max: 173, stddev: 50.740470019009805">50.97 ± 50.74</a> |

#### Text-based formats sizes

These are text-based formats, which support the full feature set of this test suite.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `musli_json`[^incomplete] | <a title="samples: 500, min: 428, max: 442, stddev: 2.370359466410104">435.41 ± 2.37</a> | <a title="samples: 500, min: 443, max: 460, stddev: 2.9921657708088594">452.29 ± 2.99</a> | <a title="samples: 10, min: 43233, max: 137190, stddev: 24215.498920319606">102385.20 ± 24215.50</a> | <a title="samples: 100, min: 635, max: 1233, stddev: 118.61131649214592">935.66 ± 118.61</a> | <a title="samples: 4000, min: 12, max: 508, stddev: 155.11569621072343">109.79 ± 155.12</a> |
| `serde_json`[^incomplete] | <a title="samples: 500, min: 428, max: 442, stddev: 2.370359466410104">435.41 ± 2.37</a> | <a title="samples: 500, min: 443, max: 460, stddev: 2.9921657708088594">452.29 ± 2.99</a> | <a title="samples: 10, min: 42978, max: 136779, stddev: 24197.177306661204">102095.30 ± 24197.18</a> | <a title="samples: 100, min: 633, max: 1231, stddev: 118.61131649214592">933.66 ± 118.61</a> | <a title="samples: 4000, min: 7, max: 508, stddev: 155.60660260232385">107.17 ± 155.61</a> |

#### Fewer features sizes

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `map` - Maps like `MashMap<K, V>` are not supported.

This is a suite where support for 128-bit integers and maps are disabled.
Usually because the underlying framework lacks support for them.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `musli_descriptive` | <a title="samples: 500, min: 112, max: 120, stddev: 1.4613363746926964">116.36 ± 1.46</a> | <a title="samples: 500, min: 118, max: 126, stddev: 1.457772273024832">122.33 ± 1.46</a> | <a title="samples: 10, min: 17864, max: 47252, stddev: 9328.948976170896">30994.00 ± 9328.95</a> | <a title="samples: 100, min: 299, max: 737, stddev: 97.85942724132408">488.45 ± 97.86</a> | <a title="samples: 4000, min: 4, max: 181, stddev: 54.38790407572287">48.29 ± 54.39</a> |
| `musli_storage` | <a title="samples: 500, min: 84, max: 91, stddev: 1.280818488311287">88.25 ± 1.28</a> | <a title="samples: 500, min: 88, max: 94, stddev: 1.2251938622112004">91.33 ± 1.23</a> | <a title="samples: 10, min: 14115, max: 37355, stddev: 7365.246537625199">24396.80 ± 7365.25</a> | <a title="samples: 100, min: 282, max: 713, stddev: 96.77495337121067">467.22 ± 96.77</a> | <a title="samples: 4000, min: 2, max: 148, stddev: 43.91472395378952">38.34 ± 43.91</a> |
| `musli_storage_packed` | <a title="samples: 500, min: 63, max: 67, stddev: 0.7069257386741584">65.98 ± 0.71</a> | <a title="samples: 500, min: 65, max: 68, stddev: 0.7482539675805259">67.05 ± 0.75</a> | <a title="samples: 10, min: 11415, max: 29917, stddev: 5835.089295803449">19503.90 ± 5835.09</a> | <a title="samples: 100, min: 264, max: 694, stddev: 96.33370074901097">448.59 ± 96.33</a> | <a title="samples: 4000, min: 2, max: 148, stddev: 39.82015650396179">34.69 ± 39.82</a> |
| `musli_wire` | <a title="samples: 500, min: 96, max: 106, stddev: 1.7524143345681649">101.86 ± 1.75</a> | <a title="samples: 500, min: 102, max: 111, stddev: 1.7655310815729104">106.83 ± 1.77</a> | <a title="samples: 10, min: 16463, max: 43841, stddev: 8662.557910917536">28513.80 ± 8662.56</a> | <a title="samples: 100, min: 288, max: 726, stddev: 98.035046794501">477.64 ± 98.04</a> | <a title="samples: 4000, min: 3, max: 173, stddev: 49.88740967418499">44.07 ± 49.89</a> |
| `serde_cbor`[^i128] | <a title="samples: 500, min: 210, max: 213, stddev: 0.5346961754117986">212.69 ± 0.53</a> | <a title="samples: 500, min: 218, max: 222, stddev: 0.847610759724064">221.17 ± 0.85</a> | <a title="samples: 10, min: 20033, max: 47027, stddev: 9429.151033364564">34759.30 ± 9429.15</a> | <a title="samples: 100, min: 380, max: 815, stddev: 97.29138656633484">566.69 ± 97.29</a> | <a title="samples: 4000, min: 6, max: 251, stddev: 80.46084400152334">65.78 ± 80.46</a> |

#### Müsli vs rkyv sizes

> **Missing features:**
> - `cstring` - `CString`'s are not supported.
> - `map` - Maps like `MashMap<K, V>` are not supported.
> - `set` - Sets like `HashSet<T>` are not supported.
> - `tuple` - Tuples like `(u32, u32)` are not supported.
> - `usize` - `usize` and `isize` types are not supported.

Comparison between [`musli-zerocopy`] and [`rkyv`].

Note that `musli-zerocopy` only supports the `primitives` benchmark.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `musli_zerocopy` | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 500, min: 80, max: 80, stddev: 0">80.00 ± 0.00</a> | - | - | - |
| `rkyv`[^incomplete] | <a title="samples: 500, min: 96, max: 48000, stddev: 13856.378747710385">24048.00 ± 13856.38</a> | <a title="samples: 500, min: 80, max: 40000, stddev: 11546.982289758653">20040.00 ± 11546.98</a> | <a title="samples: 10, min: 12016, max: 129504, stddev: 37630.54558413949">66011.20 ± 37630.55</a> | <a title="samples: 100, min: 592, max: 57216, stddev: 16294.263582058564">28866.28 ± 16294.26</a> | <a title="samples: 4000, min: 128, max: 594560, stddev: 172025.71808286753">297378.41 ± 172025.72</a> |

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
| `musli_storage` | <a title="samples: 500, min: 122, max: 131, stddev: 1.3556681009745792">127.86 ± 1.36</a> | <a title="samples: 500, min: 127, max: 134, stddev: 1.3827783625729677">130.88 ± 1.38</a> | <a title="samples: 10, min: 18742, max: 54036, stddev: 8751.096357028644">40196.50 ± 8751.10</a> | <a title="samples: 100, min: 341, max: 892, stddev: 110.04167392401843">611.30 ± 110.04</a> | <a title="samples: 4000, min: 2, max: 151, stddev: 52.9433517869571">43.30 ± 52.94</a> |
| `musli_storage_packed` | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 98, max: 100, stddev: 0.7276785004382086">99.02 ± 0.73</a> | <a title="samples: 10, min: 16303, max: 44921, stddev: 7066.113501069736">33476.30 ± 7066.11</a> | <a title="samples: 100, min: 323, max: 861, stddev: 109.30754777232909">585.80 ± 109.31</a> | <a title="samples: 4000, min: 2, max: 149, stddev: 45.8264846022475">38.70 ± 45.83</a> |
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
| `musli_storage` | <a title="samples: 500, min: 83, max: 89, stddev: 1.0542466504570922">87.24 ± 1.05</a> | <a title="samples: 500, min: 88, max: 92, stddev: 0.9941830817309317">90.34 ± 0.99</a> | <a title="samples: 10, min: 20062, max: 52778, stddev: 11415.767675018618">37789.30 ± 11415.77</a> | <a title="samples: 100, min: 350, max: 807, stddev: 101.72774007123132">570.37 ± 101.73</a> | <a title="samples: 2500, min: 2, max: 149, stddev: 46.63620403763601">44.05 ± 46.64</a> |
| `musli_storage_packed` | <a title="samples: 500, min: 63, max: 66, stddev: 0.5250676146935734">65.45 ± 0.53</a> | <a title="samples: 500, min: 65, max: 67, stddev: 0.5157363667611599">66.50 ± 0.52</a> | <a title="samples: 10, min: 17495, max: 44173, stddev: 9304.66762222058">31352.80 ± 9304.67</a> | <a title="samples: 100, min: 335, max: 791, stddev: 101.37758085494052">554.69 ± 101.38</a> | <a title="samples: 2500, min: 2, max: 149, stddev: 41.095866310859535">38.91 ± 41.10</a> |
| `musli_wire` | <a title="samples: 500, min: 95, max: 104, stddev: 1.5956490842287305">100.85 ± 1.60</a> | <a title="samples: 500, min: 101, max: 109, stddev: 1.5742934923323604">105.84 ± 1.57</a> | <a title="samples: 10, min: 22820, max: 60929, stddev: 13284.040493765442">43705.40 ± 13284.04</a> | <a title="samples: 100, min: 357, max: 820, stddev: 103.13010035872165">580.32 ± 103.13</a> | <a title="samples: 2500, min: 3, max: 179, stddev: 54.380138262420814">52.30 ± 54.38</a> |

#### Miniserde sizes

> **Missing features:**
> - `cstring` - `CString`'s are not supported.
> - `btree`
> - `map` - Maps like `MashMap<K, V>` are not supported.
> - `set` - Sets like `HashSet<T>` are not supported.
> - `nonunit-variant` - Only empty unit variants are supported.
> - `128` - 128-bit integers are not supported.
> - `char` - Character types like `char` are not supported.
> - `tuple` - Tuples like `(u32, u32)` are not supported.
> - `binary-equality` - Values are not preserved exactly when serialized and deserialized. Such as floating point values, even when they are exact.

An experimental framework which only supports JSON and a limited number of Rust types.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `miniserde` | <a title="samples: 500, min: 312, max: 326, stddev: 2.2674205609017446">319.30 ± 2.27</a> | <a title="samples: 500, min: 347, max: 361, stddev: 2.460792555255309">355.35 ± 2.46</a> | <a title="samples: 10, min: 10976, max: 31790, stddev: 5570.160141324484">21864.00 ± 5570.16</a> | <a title="samples: 100, min: 41, max: 153, stddev: 30.978720115589034">97.33 ± 30.98</a> | <a title="samples: 500, min: 7, max: 7, stddev: 0">7.00 ± 0.00</a> |
| `musli_json`[^incomplete] | <a title="samples: 500, min: 302, max: 317, stddev: 2.3087754329947305">310.67 ± 2.31</a> | <a title="samples: 500, min: 339, max: 353, stddev: 2.5256729796234514">346.68 ± 2.53</a> | <a title="samples: 10, min: 10699, max: 30951, stddev: 5418.468975642474">21287.60 ± 5418.47</a> | <a title="samples: 100, min: 41, max: 153, stddev: 30.978720115589034">97.33 ± 30.98</a> | <a title="samples: 500, min: 7, max: 7, stddev: 0">7.00 ± 0.00</a> |
| `serde_json`[^incomplete] | <a title="samples: 500, min: 302, max: 317, stddev: 2.3087754329947305">310.67 ± 2.31</a> | <a title="samples: 500, min: 339, max: 353, stddev: 2.5256729796234514">346.68 ± 2.53</a> | <a title="samples: 10, min: 10699, max: 30951, stddev: 5418.468975642474">21287.60 ± 5418.47</a> | <a title="samples: 100, min: 41, max: 153, stddev: 30.978720115589034">97.33 ± 30.98</a> | <a title="samples: 500, min: 7, max: 7, stddev: 0">7.00 ± 0.00</a> |


[^bson]: BSON does not support serializing directly in-place [without patches](https://github.com/mongodb/bson-rust/pull/328). As a result it is expected to be much slower.
[^i128]: Lacks 128-bit support.
[^incomplete]: These formats do not support a wide range of Rust types. Exact level of support varies. But from a size perspective it makes size comparisons either unfair or simply an esoteric exercise since they can (or cannot) make stricter assumptions as a result.
[^musli_value]: `musli-value` is a heap-allocated, in-memory format. Deserialization is expected to be as fast as a dynamic in-memory structure can be traversed, but serialization requires a lot of allocations. It is only included for reference.
[`rkyv`]: https://docs.rs/rkyv
[`zerocopy`]: https://docs.rs/zerocopy
[`musli-zerocopy`]: https://docs.rs/musli-zerocopy
