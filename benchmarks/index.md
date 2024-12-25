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
| `dec/primitives/musli_descriptive` | **703.53ns** ± 0.71ns | 702.14ns &mdash; 704.95ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **639.07ns** ± 0.84ns | 637.59ns &mdash; 640.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **99.02ns** ± 0.15ns | 98.76ns &mdash; 99.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_value`[^musli_value] | **410.08ns** ± 0.35ns | 409.42ns &mdash; 410.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **700.89ns** ± 2.49ns | 697.65ns &mdash; 706.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_wire/report/) |
| `dec/primitives/postcard` | **269.47ns** ± 0.28ns | 268.93ns &mdash; 270.05ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/postcard/report/) |
| `dec/primitives/serde_bincode` | **135.85ns** ± 0.14ns | 135.59ns &mdash; 136.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bincode/report/) |
| `dec/primitives/serde_bitcode` | **1.27μs** ± 1.37ns | 1.27μs &mdash; 1.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bitcode/report/) |
| `dec/primitives/serde_rmp` | **320.39ns** ± 0.34ns | 319.77ns &mdash; 321.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_rmp/report/) |

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
| `enc/primitives/musli_descriptive` | **864.15ns** ± 1.36ns | 861.61ns &mdash; 866.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **723.06ns** ± 1.09ns | 721.00ns &mdash; 725.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **128.34ns** ± 0.09ns | 128.17ns &mdash; 128.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.11μs** ± 1.02ns | 1.11μs &mdash; 1.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **745.91ns** ± 1.13ns | 743.92ns &mdash; 748.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_wire/report/) |
| `enc/primitives/postcard` | **447.73ns** ± 0.38ns | 447.01ns &mdash; 448.50ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/postcard/report/) |
| `enc/primitives/serde_bincode` | **115.08ns** ± 0.14ns | 114.83ns &mdash; 115.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bincode/report/) |
| `enc/primitives/serde_bitcode` | **3.78μs** ± 4.28ns | 3.77μs &mdash; 3.79μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bitcode/report/) |
| `enc/primitives/serde_rmp` | **266.28ns** ± 0.19ns | 265.93ns &mdash; 266.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_rmp/report/) |


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
| `dec/primpacked/musli_descriptive` | **729.43ns** ± 0.81ns | 727.95ns &mdash; 731.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **661.11ns** ± 0.65ns | 659.86ns &mdash; 662.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **91.63ns** ± 0.17ns | 91.36ns &mdash; 92.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **459.56ns** ± 0.48ns | 458.73ns &mdash; 460.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **730.98ns** ± 0.60ns | 729.80ns &mdash; 732.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/postcard` | **265.14ns** ± 0.24ns | 264.74ns &mdash; 265.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/postcard/report/) |
| `dec/primpacked/serde_bincode` | **104.71ns** ± 0.14ns | 104.46ns &mdash; 104.99ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_bincode/report/) |
| `dec/primpacked/serde_bitcode` | **1.48μs** ± 2.19ns | 1.48μs &mdash; 1.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_bitcode/report/) |
| `dec/primpacked/serde_rmp` | **397.15ns** ± 0.33ns | 396.52ns &mdash; 397.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_rmp/report/) |

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
| `enc/primpacked/musli_descriptive` | **777.09ns** ± 0.99ns | 775.27ns &mdash; 779.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **664.11ns** ± 0.72ns | 662.69ns &mdash; 665.51ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **114.18ns** ± 0.13ns | 113.94ns &mdash; 114.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **1.37μs** ± 1.52ns | 1.37μs &mdash; 1.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **659.12ns** ± 0.62ns | 657.92ns &mdash; 660.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/postcard` | **426.27ns** ± 0.40ns | 425.61ns &mdash; 427.16ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/postcard/report/) |
| `enc/primpacked/serde_bincode` | **126.85ns** ± 0.08ns | 126.69ns &mdash; 127.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_bincode/report/) |
| `enc/primpacked/serde_bitcode` | **4.64μs** ± 3.51ns | 4.63μs &mdash; 4.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_bitcode/report/) |
| `enc/primpacked/serde_rmp` | **326.11ns** ± 0.36ns | 325.44ns &mdash; 326.86ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_rmp/report/) |


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
| `dec/medium_enum/musli_descriptive` | **2.04μs** ± 2.20ns | 2.03μs &mdash; 2.04μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.45μs** ± 1.34ns | 1.45μs &mdash; 1.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **826.19ns** ± 1.78ns | 823.25ns &mdash; 830.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **1.01μs** ± 1.11ns | 1.00μs &mdash; 1.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **1.67μs** ± 1.76ns | 1.66μs &mdash; 1.67μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/postcard` | **1.19μs** ± 1.63ns | 1.19μs &mdash; 1.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/postcard/report/) |
| `dec/medium_enum/serde_bincode` | **939.98ns** ± 1.19ns | 937.80ns &mdash; 942.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_bincode/report/) |
| `dec/medium_enum/serde_bitcode` | **9.27μs** ± 13.26ns | 9.25μs &mdash; 9.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_bitcode/report/) |
| `dec/medium_enum/serde_rmp` | **2.37μs** ± 2.11ns | 2.36μs &mdash; 2.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_rmp/report/) |

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
| `enc/medium_enum/musli_descriptive` | **1.51μs** ± 2.41ns | 1.51μs &mdash; 1.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **1.17μs** ± 1.78ns | 1.17μs &mdash; 1.17μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **421.58ns** ± 0.30ns | 421.01ns &mdash; 422.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **3.12μs** ± 3.45ns | 3.11μs &mdash; 3.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **1.25μs** ± 1.32ns | 1.25μs &mdash; 1.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/postcard` | **893.50ns** ± 1.21ns | 891.19ns &mdash; 895.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/postcard/report/) |
| `enc/medium_enum/serde_bincode` | **316.68ns** ± 0.28ns | 316.21ns &mdash; 317.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_bincode/report/) |
| `enc/medium_enum/serde_bitcode` | **13.08μs** ± 11.07ns | 13.06μs &mdash; 13.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_bitcode/report/) |
| `enc/medium_enum/serde_rmp` | **714.70ns** ± 0.88ns | 713.17ns &mdash; 716.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_rmp/report/) |


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
| `dec/large/musli_descriptive` | **286.77μs** ± 257.93ns | 286.31μs &mdash; 287.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **190.00μs** ± 241.06ns | 189.58μs &mdash; 190.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **102.12μs** ± 131.26ns | 101.89μs &mdash; 102.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_value`[^musli_value] | **127.26μs** ± 354.88ns | 126.69μs &mdash; 128.06μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **224.11μs** ± 224.11ns | 223.73μs &mdash; 224.60μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_wire/report/) |
| `dec/large/postcard` | **89.96μs** ± 90.82ns | 89.80μs &mdash; 90.15μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/postcard/report/) |
| `dec/large/serde_bincode` | **68.25μs** ± 45.41ns | 68.16μs &mdash; 68.34μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bincode/report/) |
| `dec/large/serde_bitcode` | **103.01μs** ± 200.12ns | 102.66μs &mdash; 103.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bitcode/report/) |
| `dec/large/serde_rmp` | **222.55μs** ± 189.97ns | 222.22μs &mdash; 222.96μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_rmp/report/) |

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
| `enc/large/musli_descriptive` | **170.65μs** ± 199.99ns | 170.29μs &mdash; 171.07μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **136.43μs** ± 118.69ns | 136.22μs &mdash; 136.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **46.45μs** ± 34.19ns | 46.38μs &mdash; 46.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_value`[^musli_value] | **713.63μs** ± 616.93ns | 712.48μs &mdash; 714.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **147.33μs** ± 170.03ns | 147.03μs &mdash; 147.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_wire/report/) |
| `enc/large/postcard` | **113.99μs** ± 339.19ns | 113.42μs &mdash; 114.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/postcard/report/) |
| `enc/large/serde_bincode` | **42.51μs** ± 32.98ns | 42.45μs &mdash; 42.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bincode/report/) |
| `enc/large/serde_bitcode` | **107.94μs** ± 145.41ns | 107.67μs &mdash; 108.24μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bitcode/report/) |
| `enc/large/serde_rmp` | **156.08μs** ± 126.97ns | 155.85μs &mdash; 156.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_rmp/report/) |


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
| `dec/allocated/musli_descriptive` | **3.32μs** ± 2.80ns | 3.32μs &mdash; 3.33μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **3.10μs** ± 3.77ns | 3.09μs &mdash; 3.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.56μs** ± 2.98ns | 2.56μs &mdash; 2.57μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_value`[^musli_value] | **2.08μs** ± 1.64ns | 2.08μs &mdash; 2.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **3.21μs** ± 5.51ns | 3.20μs &mdash; 3.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_wire/report/) |
| `dec/allocated/postcard` | **3.50μs** ± 2.74ns | 3.50μs &mdash; 3.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/postcard/report/) |
| `dec/allocated/serde_bincode` | **3.23μs** ± 3.99ns | 3.22μs &mdash; 3.23μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bincode/report/) |
| `dec/allocated/serde_bitcode` | **5.88μs** ± 6.57ns | 5.87μs &mdash; 5.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bitcode/report/) |
| `dec/allocated/serde_rmp` | **4.23μs** ± 3.87ns | 4.22μs &mdash; 4.24μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_rmp/report/) |

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
| `enc/allocated/musli_descriptive` | **747.61ns** ± 0.94ns | 745.85ns &mdash; 749.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **675.71ns** ± 0.69ns | 674.46ns &mdash; 677.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **320.19ns** ± 0.32ns | 319.65ns &mdash; 320.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_value`[^musli_value] | **2.42μs** ± 2.42ns | 2.41μs &mdash; 2.42μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **626.58ns** ± 0.78ns | 625.12ns &mdash; 628.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_wire/report/) |
| `enc/allocated/postcard` | **1.21μs** ± 0.88ns | 1.21μs &mdash; 1.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/postcard/report/) |
| `enc/allocated/serde_bincode` | **401.17ns** ± 0.30ns | 400.61ns &mdash; 401.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bincode/report/) |
| `enc/allocated/serde_bitcode` | **8.13μs** ± 7.78ns | 8.12μs &mdash; 8.15μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bitcode/report/) |
| `enc/allocated/serde_rmp` | **776.84ns** ± 0.85ns | 775.31ns &mdash; 778.64ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_rmp/report/) |



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
| `dec/primitives/musli_json` | **3.74μs** ± 4.54ns | 3.73μs &mdash; 3.74μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **4.42μs** ± 4.54ns | 4.41μs &mdash; 4.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/serde_json/report/) |

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
| `enc/primitives/musli_json` | **1.31μs** ± 1.40ns | 1.31μs &mdash; 1.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **1.32μs** ± 1.45ns | 1.31μs &mdash; 1.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/serde_json/report/) |


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
| `dec/primpacked/musli_json` | **4.30μs** ± 4.51ns | 4.29μs &mdash; 4.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/musli_json/report/) |
| `dec/primpacked/serde_json` | **4.67μs** ± 5.46ns | 4.66μs &mdash; 4.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/serde_json/report/) |

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
| `enc/primpacked/musli_json` | **1.20μs** ± 0.85ns | 1.20μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/musli_json/report/) |
| `enc/primpacked/serde_json` | **1.40μs** ± 1.50ns | 1.40μs &mdash; 1.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/serde_json/report/) |


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
| `dec/medium_enum/musli_json` | **8.75μs** ± 8.21ns | 8.73μs &mdash; 8.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/musli_json/report/) |
| `dec/medium_enum/serde_json` | **8.29μs** ± 6.02ns | 8.28μs &mdash; 8.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/serde_json/report/) |

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
| `enc/medium_enum/musli_json` | **2.69μs** ± 2.07ns | 2.69μs &mdash; 2.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/musli_json/report/) |
| `enc/medium_enum/serde_json` | **2.57μs** ± 5.30ns | 2.56μs &mdash; 2.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/serde_json/report/) |


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
| `dec/large/musli_json` | **997.42μs** ± 1.47μs | 994.86μs &mdash; 1.00ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **773.93μs** ± 1.07μs | 772.15μs &mdash; 776.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/serde_json/report/) |

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
| `enc/large/musli_json` | **297.40μs** ± 422.21ns | 296.69μs &mdash; 298.33μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **303.94μs** ± 270.20ns | 303.48μs &mdash; 304.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/serde_json/report/) |


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
| `dec/allocated/musli_json` | **9.83μs** ± 12.22ns | 9.81μs &mdash; 9.86μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **7.98μs** ± 15.41ns | 7.95μs &mdash; 8.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/serde_json/report/) |

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
| `enc/allocated/musli_json` | **2.32μs** ± 1.88ns | 2.32μs &mdash; 2.33μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **2.50μs** ± 2.00ns | 2.50μs &mdash; 2.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/serde_json/report/) |



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
| `dec/primitives/musli_descriptive` | **527.63ns** ± 1.01ns | 525.98ns &mdash; 529.87ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **485.30ns** ± 0.73ns | 483.95ns &mdash; 486.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **76.27ns** ± 0.09ns | 76.11ns &mdash; 76.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_value`[^musli_value] | **350.03ns** ± 0.40ns | 349.34ns &mdash; 350.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **542.91ns** ± 0.48ns | 542.00ns &mdash; 543.88ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_wire/report/) |
| `dec/primitives/serde_cbor` | **1.66μs** ± 2.06ns | 1.66μs &mdash; 1.67μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/serde_cbor/report/) |

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
| `enc/primitives/musli_descriptive` | **495.98ns** ± 0.79ns | 494.47ns &mdash; 497.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **393.75ns** ± 0.29ns | 393.21ns &mdash; 394.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **109.39ns** ± 0.09ns | 109.24ns &mdash; 109.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.02μs** ± 0.76ns | 1.01μs &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **387.99ns** ± 0.68ns | 386.69ns &mdash; 389.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_wire/report/) |
| `enc/primitives/serde_cbor` | **430.78ns** ± 0.39ns | 430.11ns &mdash; 431.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/serde_cbor/report/) |


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
| `dec/primpacked/musli_descriptive` | **579.05ns** ± 0.56ns | 578.03ns &mdash; 580.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **504.35ns** ± 0.50ns | 503.45ns &mdash; 505.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **68.58ns** ± 0.08ns | 68.45ns &mdash; 68.75ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **366.81ns** ± 0.53ns | 365.96ns &mdash; 368.00ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **566.22ns** ± 0.66ns | 565.07ns &mdash; 567.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/serde_cbor` | **1.77μs** ± 2.06ns | 1.77μs &mdash; 1.78μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/serde_cbor/report/) |

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
| `enc/primpacked/musli_descriptive` | **472.50ns** ± 0.54ns | 471.48ns &mdash; 473.60ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **339.31ns** ± 0.57ns | 338.21ns &mdash; 340.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **105.53ns** ± 0.11ns | 105.33ns &mdash; 105.76ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **1.19μs** ± 1.10ns | 1.19μs &mdash; 1.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **340.79ns** ± 0.56ns | 339.71ns &mdash; 341.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/serde_cbor` | **487.01ns** ± 0.49ns | 486.15ns &mdash; 488.05ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/serde_cbor/report/) |


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
| `dec/medium_enum/musli_descriptive` | **1.91μs** ± 2.80ns | 1.91μs &mdash; 1.92μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.26μs** ± 1.54ns | 1.26μs &mdash; 1.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **801.41ns** ± 0.95ns | 799.73ns &mdash; 803.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **977.02ns** ± 1.14ns | 975.00ns &mdash; 979.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **1.50μs** ± 1.51ns | 1.50μs &mdash; 1.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/serde_cbor` | **4.61μs** ± 5.71ns | 4.60μs &mdash; 4.63μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/serde_cbor/report/) |

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
| `enc/medium_enum/musli_descriptive` | **1.15μs** ± 1.60ns | 1.14μs &mdash; 1.15μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **839.65ns** ± 0.95ns | 837.90ns &mdash; 841.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **424.78ns** ± 0.50ns | 423.86ns &mdash; 425.84ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **3.20μs** ± 4.33ns | 3.19μs &mdash; 3.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **895.23ns** ± 0.97ns | 893.53ns &mdash; 897.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/serde_cbor` | **1.03μs** ± 1.48ns | 1.03μs &mdash; 1.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/serde_cbor/report/) |


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
| `dec/large/musli_descriptive` | **311.61μs** ± 260.19ns | 311.15μs &mdash; 312.17μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **197.28μs** ± 159.89ns | 197.00μs &mdash; 197.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **99.29μs** ± 114.87ns | 99.09μs &mdash; 99.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_value`[^musli_value] | **137.80μs** ± 201.01ns | 137.41μs &mdash; 138.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **236.33μs** ± 263.27ns | 235.85μs &mdash; 236.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_wire/report/) |
| `dec/large/serde_cbor` | **578.90μs** ± 467.31ns | 578.03μs &mdash; 579.86μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/serde_cbor/report/) |

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
| `enc/large/musli_descriptive` | **186.76μs** ± 155.07ns | 186.48μs &mdash; 187.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **145.61μs** ± 152.20ns | 145.34μs &mdash; 145.93μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **51.68μs** ± 36.24ns | 51.62μs &mdash; 51.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_value`[^musli_value] | **763.25μs** ± 1.21μs | 761.19μs &mdash; 765.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **156.52μs** ± 184.39ns | 156.21μs &mdash; 156.92μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_wire/report/) |
| `enc/large/serde_cbor` | **172.33μs** ± 179.55ns | 172.03μs &mdash; 172.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/serde_cbor/report/) |


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
| `dec/allocated/musli_descriptive` | **2.33μs** ± 3.46ns | 2.32μs &mdash; 2.34μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **2.23μs** ± 2.89ns | 2.22μs &mdash; 2.24μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **1.89μs** ± 1.66ns | 1.88μs &mdash; 1.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_value`[^musli_value] | **1.45μs** ± 1.65ns | 1.45μs &mdash; 1.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **2.40μs** ± 2.39ns | 2.40μs &mdash; 2.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_wire/report/) |
| `dec/allocated/serde_cbor` | **4.87μs** ± 6.18ns | 4.85μs &mdash; 4.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/serde_cbor/report/) |

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
| `enc/allocated/musli_descriptive` | **540.51ns** ± 0.57ns | 539.41ns &mdash; 541.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **480.06ns** ± 0.62ns | 478.97ns &mdash; 481.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **249.87ns** ± 0.19ns | 249.52ns &mdash; 250.25ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_value`[^musli_value] | **1.99μs** ± 2.05ns | 1.99μs &mdash; 2.00μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **438.77ns** ± 0.36ns | 438.08ns &mdash; 439.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_wire/report/) |
| `enc/allocated/serde_cbor` | **647.09ns** ± 1.39ns | 644.76ns &mdash; 650.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/serde_cbor/report/) |



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
| `dec/primitives/musli_zerocopy` | **4.02ns** ± 0.00ns | 4.01ns &mdash; 4.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primitives/musli_zerocopy/report/) |
| `dec/primitives/rkyv` | **14.69ns** ± 0.02ns | 14.66ns &mdash; 14.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primitives/rkyv/report/) |

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
| `enc/primitives/musli_zerocopy` | **20.04ns** ± 0.02ns | 20.00ns &mdash; 20.08ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primitives/musli_zerocopy/report/) |
| `enc/primitives/rkyv` | **33.03ns** ± 0.04ns | 32.96ns &mdash; 33.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primitives/rkyv/report/) |


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
| `dec/primpacked/musli_zerocopy` | **2.66ns** ± 0.00ns | 2.66ns &mdash; 2.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primpacked/musli_zerocopy/report/) |
| `dec/primpacked/rkyv` | **14.17ns** ± 0.01ns | 14.15ns &mdash; 14.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primpacked/rkyv/report/) |

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
| `enc/primpacked/musli_zerocopy` | **16.91ns** ± 0.02ns | 16.88ns &mdash; 16.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primpacked/musli_zerocopy/report/) |
| `enc/primpacked/rkyv` | **32.47ns** ± 0.03ns | 32.41ns &mdash; 32.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primpacked/rkyv/report/) |



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
| `dec/primpacked/musli_zerocopy` | **2.66ns** ± 0.00ns | 2.66ns &mdash; 2.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/musli_zerocopy/report/) |
| `dec/primpacked/zerocopy` | **6.64ns** ± 0.00ns | 6.63ns &mdash; 6.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/zerocopy/report/) |

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
| `enc/primpacked/musli_zerocopy` | **17.87ns** ± 0.01ns | 17.84ns &mdash; 17.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/musli_zerocopy/report/) |
| `enc/primpacked/zerocopy` | **8.43ns** ± 0.01ns | 8.42ns &mdash; 8.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/zerocopy/report/) |



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
| `dec/primitives/derive_bitcode` | **249.47ns** ± 0.32ns | 248.89ns &mdash; 250.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/derive_bitcode/report/) |
| `dec/primitives/musli_descriptive` | **699.63ns** ± 0.75ns | 698.22ns &mdash; 701.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **633.63ns** ± 0.75ns | 632.28ns &mdash; 635.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **98.94ns** ± 0.10ns | 98.77ns &mdash; 99.16ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_wire` | **697.28ns** ± 0.96ns | 695.53ns &mdash; 699.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/derive_bitcode` | **1.30μs** ± 1.41ns | 1.29μs &mdash; 1.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/derive_bitcode/report/) |
| `enc/primitives/musli_descriptive` | **866.98ns** ± 1.06ns | 864.95ns &mdash; 869.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **721.15ns** ± 0.77ns | 719.68ns &mdash; 722.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **118.83ns** ± 0.14ns | 118.57ns &mdash; 119.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_wire` | **705.23ns** ± 0.89ns | 703.62ns &mdash; 707.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_wire/report/) |


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
| `dec/primpacked/derive_bitcode` | **249.98ns** ± 0.21ns | 249.61ns &mdash; 250.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/derive_bitcode/report/) |
| `dec/primpacked/musli_descriptive` | **717.06ns** ± 0.68ns | 715.79ns &mdash; 718.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **658.35ns** ± 0.82ns | 656.84ns &mdash; 660.05ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **91.63ns** ± 0.09ns | 91.47ns &mdash; 91.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_wire` | **729.34ns** ± 0.73ns | 728.08ns &mdash; 730.94ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_wire/report/) |

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
| `enc/primpacked/derive_bitcode` | **1.30μs** ± 1.26ns | 1.30μs &mdash; 1.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/derive_bitcode/report/) |
| `enc/primpacked/musli_descriptive` | **779.89ns** ± 0.96ns | 778.10ns &mdash; 781.87ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **675.31ns** ± 0.72ns | 674.01ns &mdash; 676.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **113.83ns** ± 0.19ns | 113.49ns &mdash; 114.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_wire` | **661.99ns** ± 1.04ns | 660.04ns &mdash; 664.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_wire/report/) |


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
| `dec/medium_enum/derive_bitcode` | **3.24μs** ± 3.69ns | 3.24μs &mdash; 3.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/derive_bitcode/report/) |
| `dec/medium_enum/musli_descriptive` | **2.19μs** ± 1.84ns | 2.19μs &mdash; 2.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.64μs** ± 2.19ns | 1.64μs &mdash; 1.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **1.03μs** ± 0.80ns | 1.03μs &mdash; 1.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_wire` | **1.83μs** ± 2.06ns | 1.83μs &mdash; 1.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_wire/report/) |

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
| `enc/medium_enum/derive_bitcode` | **13.47μs** ± 11.54ns | 13.45μs &mdash; 13.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/derive_bitcode/report/) |
| `enc/medium_enum/musli_descriptive` | **1.48μs** ± 1.21ns | 1.48μs &mdash; 1.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **1.17μs** ± 0.97ns | 1.17μs &mdash; 1.17μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **417.89ns** ± 0.33ns | 417.25ns &mdash; 418.56ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_wire` | **1.23μs** ± 1.28ns | 1.23μs &mdash; 1.24μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_wire/report/) |


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
| `dec/large/derive_bitcode` | **32.20μs** ± 43.16ns | 32.12μs &mdash; 32.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/derive_bitcode/report/) |
| `dec/large/musli_descriptive` | **289.64μs** ± 302.71ns | 289.11μs &mdash; 290.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **192.07μs** ± 250.94ns | 191.63μs &mdash; 192.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **99.94μs** ± 90.84ns | 99.78μs &mdash; 100.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_wire` | **227.40μs** ± 249.26ns | 226.96μs &mdash; 227.93μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_wire/report/) |

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
| `enc/large/derive_bitcode` | **86.27μs** ± 133.00ns | 86.05μs &mdash; 86.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/derive_bitcode/report/) |
| `enc/large/musli_descriptive` | **169.72μs** ± 144.36ns | 169.46μs &mdash; 170.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **136.63μs** ± 111.87ns | 136.43μs &mdash; 136.87μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **46.29μs** ± 40.96ns | 46.22μs &mdash; 46.38μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_wire` | **156.60μs** ± 158.14ns | 156.31μs &mdash; 156.93μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_wire/report/) |


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
| `dec/allocated/derive_bitcode` | **3.82μs** ± 5.03ns | 3.81μs &mdash; 3.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/derive_bitcode/report/) |
| `dec/allocated/musli_descriptive` | **3.69μs** ± 4.78ns | 3.68μs &mdash; 3.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **3.51μs** ± 4.65ns | 3.51μs &mdash; 3.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **3.00μs** ± 2.20ns | 2.99μs &mdash; 3.00μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_wire` | **3.66μs** ± 3.23ns | 3.65μs &mdash; 3.67μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_wire/report/) |

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
| `enc/allocated/derive_bitcode` | **7.26μs** ± 6.91ns | 7.25μs &mdash; 7.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/derive_bitcode/report/) |
| `enc/allocated/musli_descriptive` | **698.15ns** ± 0.69ns | 696.85ns &mdash; 699.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **657.30ns** ± 1.08ns | 655.38ns &mdash; 659.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **310.11ns** ± 0.32ns | 309.54ns &mdash; 310.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_wire` | **622.86ns** ± 0.68ns | 621.58ns &mdash; 624.24ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_wire/report/) |



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
| `dec/primitives/bson`[^bson] | **2.87μs** ± 3.64ns | 2.86μs &mdash; 2.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/bson/report/) |
| `dec/primitives/musli_descriptive` | **543.16ns** ± 0.50ns | 542.22ns &mdash; 544.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **466.43ns** ± 0.66ns | 465.23ns &mdash; 467.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **75.31ns** ± 0.08ns | 75.18ns &mdash; 75.48ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_wire` | **519.57ns** ± 0.46ns | 518.71ns &mdash; 520.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/bson`[^bson] | **1.36μs** ± 0.98ns | 1.36μs &mdash; 1.36μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/bson/report/) |
| `enc/primitives/musli_descriptive` | **484.63ns** ± 0.38ns | 483.93ns &mdash; 485.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **374.52ns** ± 0.42ns | 373.77ns &mdash; 375.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **99.60ns** ± 0.09ns | 99.43ns &mdash; 99.79ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_wire` | **344.20ns** ± 0.44ns | 343.51ns &mdash; 345.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_wire/report/) |


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
| `dec/primpacked/bson`[^bson] | **3.87μs** ± 3.69ns | 3.87μs &mdash; 3.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/bson/report/) |
| `dec/primpacked/musli_descriptive` | **572.59ns** ± 0.55ns | 571.57ns &mdash; 573.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **502.23ns** ± 0.45ns | 501.38ns &mdash; 503.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **67.41ns** ± 0.07ns | 67.29ns &mdash; 67.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_wire` | **550.30ns** ± 0.54ns | 549.33ns &mdash; 551.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_wire/report/) |

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
| `enc/primpacked/bson`[^bson] | **2.47μs** ± 2.41ns | 2.46μs &mdash; 2.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/bson/report/) |
| `enc/primpacked/musli_descriptive` | **437.05ns** ± 0.76ns | 435.61ns &mdash; 438.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **342.33ns** ± 0.63ns | 341.10ns &mdash; 343.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **98.58ns** ± 0.14ns | 98.33ns &mdash; 98.86ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_wire` | **319.83ns** ± 0.48ns | 318.91ns &mdash; 320.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_wire/report/) |


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
| `dec/medium_enum/bson`[^bson] | **7.94μs** ± 7.81ns | 7.93μs &mdash; 7.96μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/bson/report/) |
| `dec/medium_enum/musli_descriptive` | **1.58μs** ± 1.90ns | 1.58μs &mdash; 1.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **997.78ns** ± 2.02ns | 994.12ns &mdash; 1.00μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **570.58ns** ± 0.60ns | 569.51ns &mdash; 571.84ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_wire` | **1.18μs** ± 1.79ns | 1.18μs &mdash; 1.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_wire/report/) |

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
| `enc/medium_enum/bson`[^bson] | **5.27μs** ± 4.29ns | 5.26μs &mdash; 5.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/bson/report/) |
| `enc/medium_enum/musli_descriptive` | **972.16ns** ± 0.96ns | 970.42ns &mdash; 974.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **699.74ns** ± 1.19ns | 697.59ns &mdash; 702.28ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **323.93ns** ± 0.40ns | 323.21ns &mdash; 324.75ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_wire` | **747.26ns** ± 0.81ns | 745.92ns &mdash; 749.04ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_wire/report/) |


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
| `dec/large/bson`[^bson] | **1.79ms** ± 1.18μs | 1.78ms &mdash; 1.79ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/bson/report/) |
| `dec/large/musli_descriptive` | **385.99μs** ± 436.40ns | 385.19μs &mdash; 386.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **252.37μs** ± 317.81ns | 251.79μs &mdash; 253.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **140.03μs** ± 148.42ns | 139.76μs &mdash; 140.34μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_wire` | **298.38μs** ± 236.70ns | 297.95μs &mdash; 298.87μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_wire/report/) |

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
| `enc/large/bson`[^bson] | **980.20μs** ± 664.55ns | 978.97μs &mdash; 981.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/bson/report/) |
| `enc/large/musli_descriptive` | **200.13μs** ± 265.52ns | 199.65μs &mdash; 200.69μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **157.05μs** ± 122.18ns | 156.83μs &mdash; 157.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **59.73μs** ± 91.93ns | 59.57μs &mdash; 59.93μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_wire` | **163.84μs** ± 172.59ns | 163.53μs &mdash; 164.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_wire/report/) |


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
| `dec/allocated/bson`[^bson] | **7.67μs** ± 10.89ns | 7.65μs &mdash; 7.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/bson/report/) |
| `dec/allocated/musli_descriptive` | **2.98μs** ± 4.10ns | 2.97μs &mdash; 2.99μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **2.94μs** ± 2.69ns | 2.93μs &mdash; 2.94μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.60μs** ± 2.28ns | 2.60μs &mdash; 2.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_wire` | **3.02μs** ± 3.71ns | 3.02μs &mdash; 3.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_wire/report/) |

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
| `enc/allocated/bson`[^bson] | **2.47μs** ± 3.05ns | 2.47μs &mdash; 2.48μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/bson/report/) |
| `enc/allocated/musli_descriptive` | **471.83ns** ± 0.51ns | 470.89ns &mdash; 472.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **419.34ns** ± 0.37ns | 418.70ns &mdash; 420.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **252.02ns** ± 0.24ns | 251.58ns &mdash; 252.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_wire` | **361.34ns** ± 0.47ns | 360.50ns &mdash; 362.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_wire/report/) |



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
| `dec/primitives/miniserde` | **2.12μs** ± 1.82ns | 2.12μs &mdash; 2.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/miniserde/report/) |
| `dec/primitives/musli_json` | **2.53μs** ± 2.52ns | 2.53μs &mdash; 2.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **2.22μs** ± 2.62ns | 2.22μs &mdash; 2.23μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/serde_json/report/) |

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
| `enc/primitives/miniserde` | **2.45μs** ± 3.18ns | 2.44μs &mdash; 2.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/miniserde/report/) |
| `enc/primitives/musli_json` | **958.02ns** ± 1.20ns | 955.92ns &mdash; 960.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **974.55ns** ± 1.89ns | 970.95ns &mdash; 978.33ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/serde_json/report/) |


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
| `dec/primpacked/miniserde` | **2.84μs** ± 3.35ns | 2.83μs &mdash; 2.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/miniserde/report/) |
| `dec/primpacked/musli_json` | **3.42μs** ± 3.80ns | 3.42μs &mdash; 3.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/musli_json/report/) |
| `dec/primpacked/serde_json` | **2.82μs** ± 4.88ns | 2.81μs &mdash; 2.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/serde_json/report/) |

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
| `enc/primpacked/miniserde` | **2.99μs** ± 3.79ns | 2.98μs &mdash; 3.00μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/miniserde/report/) |
| `enc/primpacked/musli_json` | **902.35ns** ± 0.70ns | 901.07ns &mdash; 903.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/musli_json/report/) |
| `enc/primpacked/serde_json` | **1.12μs** ± 0.96ns | 1.12μs &mdash; 1.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/serde_json/report/) |


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
| `dec/medium_enum/miniserde` | **67.62ns** ± 0.07ns | 67.49ns &mdash; 67.76ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/miniserde/report/) |
| `dec/medium_enum/musli_json` | **64.02ns** ± 0.08ns | 63.88ns &mdash; 64.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/musli_json/report/) |
| `dec/medium_enum/serde_json` | **73.41ns** ± 0.07ns | 73.29ns &mdash; 73.56ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/serde_json/report/) |

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
| `enc/medium_enum/miniserde` | **93.83ns** ± 0.09ns | 93.65ns &mdash; 94.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/miniserde/report/) |
| `enc/medium_enum/musli_json` | **24.01ns** ± 0.02ns | 23.97ns &mdash; 24.05ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/musli_json/report/) |
| `enc/medium_enum/serde_json` | **28.76ns** ± 0.02ns | 28.73ns &mdash; 28.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/serde_json/report/) |


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
| `dec/large/miniserde` | **189.20μs** ± 216.17ns | 188.80μs &mdash; 189.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/miniserde/report/) |
| `dec/large/musli_json` | **245.94μs** ± 264.33ns | 245.47μs &mdash; 246.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **220.61μs** ± 248.19ns | 220.17μs &mdash; 221.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/serde_json/report/) |

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
| `enc/large/miniserde` | **153.29μs** ± 146.34ns | 153.03μs &mdash; 153.60μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/miniserde/report/) |
| `enc/large/musli_json` | **93.70μs** ± 91.06ns | 93.54μs &mdash; 93.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **93.75μs** ± 97.53ns | 93.59μs &mdash; 93.97μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/serde_json/report/) |


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
| `dec/allocated/miniserde` | **573.76ns** ± 0.52ns | 572.79ns &mdash; 574.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/miniserde/report/) |
| `dec/allocated/musli_json` | **571.42ns** ± 0.59ns | 570.37ns &mdash; 572.68ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **395.99ns** ± 0.40ns | 395.26ns &mdash; 396.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/serde_json/report/) |

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
| `enc/allocated/miniserde` | **658.99ns** ± 1.09ns | 657.08ns &mdash; 661.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/miniserde/report/) |
| `enc/allocated/musli_json` | **135.82ns** ± 0.11ns | 135.61ns &mdash; 136.06ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **170.71ns** ± 0.24ns | 170.27ns &mdash; 171.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/serde_json/report/) |



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
| `rkyv`[^incomplete] | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 500, min: 80, max: 80, stddev: 0">80.00 ± 0.00</a> | <a title="samples: 10, min: 8272, max: 19984, stddev: 3223.554907241383">12950.40 ± 3223.55</a> | <a title="samples: 100, min: 376, max: 824, stddev: 81.00657751071822">571.88 ± 81.01</a> | <a title="samples: 4000, min: 128, max: 272, stddev: 39.42433766089168">148.64 ± 39.42</a> |

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
