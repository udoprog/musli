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
- [**Speedy**](#speedy) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/report/), [Sizes](#speedy-sizes))
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
| `dec/primitives/musli_descriptive` | **705.13ns** ± 0.68ns | 703.92ns &mdash; 706.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **620.44ns** ± 0.49ns | 619.55ns &mdash; 621.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **15.06ns** ± 0.02ns | 15.03ns &mdash; 15.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_value`[^musli_value] | **317.80ns** ± 0.61ns | 316.89ns &mdash; 319.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **578.86ns** ± 0.81ns | 577.40ns &mdash; 580.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_wire/report/) |
| `dec/primitives/postcard` | **269.58ns** ± 0.27ns | 269.16ns &mdash; 270.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/postcard/report/) |
| `dec/primitives/serde_bincode` | **131.58ns** ± 0.09ns | 131.42ns &mdash; 131.77ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bincode/report/) |
| `dec/primitives/serde_bitcode` | **1.28μs** ± 1.41ns | 1.27μs &mdash; 1.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bitcode/report/) |
| `dec/primitives/serde_rmp` | **345.50ns** ± 0.34ns | 344.89ns &mdash; 346.22ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_rmp/report/) |

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
| `enc/primitives/musli_descriptive` | **852.80ns** ± 0.91ns | 851.12ns &mdash; 854.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **735.16ns** ± 0.98ns | 733.29ns &mdash; 737.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **64.81ns** ± 0.06ns | 64.71ns &mdash; 64.94ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.11μs** ± 1.47ns | 1.10μs &mdash; 1.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **706.17ns** ± 0.75ns | 704.76ns &mdash; 707.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_wire/report/) |
| `enc/primitives/postcard` | **431.66ns** ± 0.39ns | 430.99ns &mdash; 432.49ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/postcard/report/) |
| `enc/primitives/serde_bincode` | **114.55ns** ± 0.13ns | 114.33ns &mdash; 114.85ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bincode/report/) |
| `enc/primitives/serde_bitcode` | **3.89μs** ± 11.53ns | 3.87μs &mdash; 3.92μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bitcode/report/) |
| `enc/primitives/serde_rmp` | **267.70ns** ± 0.80ns | 266.46ns &mdash; 269.51ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_rmp/report/) |


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
| `dec/primpacked/musli_descriptive` | **724.49ns** ± 0.90ns | 722.88ns &mdash; 726.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **668.17ns** ± 0.75ns | 666.75ns &mdash; 669.68ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **26.13ns** ± 0.03ns | 26.08ns &mdash; 26.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **317.33ns** ± 0.31ns | 316.80ns &mdash; 318.00ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **583.63ns** ± 0.38ns | 582.95ns &mdash; 584.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/postcard` | **262.88ns** ± 0.28ns | 262.38ns &mdash; 263.49ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/postcard/report/) |
| `dec/primpacked/serde_bincode` | **104.11ns** ± 0.14ns | 103.87ns &mdash; 104.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_bincode/report/) |
| `dec/primpacked/serde_bitcode` | **1.51μs** ± 1.75ns | 1.51μs &mdash; 1.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_bitcode/report/) |
| `dec/primpacked/serde_rmp` | **404.61ns** ± 0.39ns | 403.92ns &mdash; 405.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_rmp/report/) |

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
| `enc/primpacked/musli_descriptive` | **762.40ns** ± 0.58ns | 761.32ns &mdash; 763.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **672.17ns** ± 1.05ns | 670.23ns &mdash; 674.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **65.90ns** ± 0.05ns | 65.81ns &mdash; 66.00ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **1.46μs** ± 1.45ns | 1.46μs &mdash; 1.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **652.75ns** ± 0.76ns | 651.31ns &mdash; 654.27ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/postcard` | **432.03ns** ± 0.38ns | 431.36ns &mdash; 432.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/postcard/report/) |
| `enc/primpacked/serde_bincode` | **126.63ns** ± 0.13ns | 126.41ns &mdash; 126.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_bincode/report/) |
| `enc/primpacked/serde_bitcode` | **4.47μs** ± 11.24ns | 4.46μs &mdash; 4.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_bitcode/report/) |
| `enc/primpacked/serde_rmp` | **327.06ns** ± 0.32ns | 326.49ns &mdash; 327.74ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_rmp/report/) |


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
| `dec/medium_enum/musli_descriptive` | **1.64μs** ± 1.44ns | 1.63μs &mdash; 1.64μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.44μs** ± 2.16ns | 1.44μs &mdash; 1.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **486.51ns** ± 0.47ns | 485.63ns &mdash; 487.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **821.66ns** ± 0.63ns | 820.51ns &mdash; 822.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **1.38μs** ± 1.13ns | 1.38μs &mdash; 1.38μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/postcard` | **1.18μs** ± 1.42ns | 1.18μs &mdash; 1.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/postcard/report/) |
| `dec/medium_enum/serde_bincode` | **945.97ns** ± 0.81ns | 944.49ns &mdash; 947.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_bincode/report/) |
| `dec/medium_enum/serde_bitcode` | **9.49μs** ± 13.96ns | 9.47μs &mdash; 9.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_bitcode/report/) |
| `dec/medium_enum/serde_rmp` | **2.32μs** ± 2.16ns | 2.31μs &mdash; 2.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_rmp/report/) |

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
| `enc/medium_enum/musli_descriptive` | **1.46μs** ± 1.01ns | 1.46μs &mdash; 1.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **1.19μs** ± 1.07ns | 1.18μs &mdash; 1.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **271.40ns** ± 0.36ns | 270.75ns &mdash; 272.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **3.17μs** ± 3.62ns | 3.16μs &mdash; 3.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **1.19μs** ± 1.91ns | 1.19μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/postcard` | **896.65ns** ± 1.15ns | 894.69ns &mdash; 899.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/postcard/report/) |
| `enc/medium_enum/serde_bincode` | **314.74ns** ± 0.31ns | 314.19ns &mdash; 315.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_bincode/report/) |
| `enc/medium_enum/serde_bitcode` | **12.86μs** ± 17.91ns | 12.83μs &mdash; 12.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_bitcode/report/) |
| `enc/medium_enum/serde_rmp` | **722.79ns** ± 1.18ns | 720.84ns &mdash; 725.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_rmp/report/) |


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
| `dec/large/musli_descriptive` | **241.21μs** ± 248.53ns | 240.78μs &mdash; 241.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **192.22μs** ± 325.19ns | 191.66μs &mdash; 192.93μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **50.92μs** ± 55.35ns | 50.82μs &mdash; 51.04μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_value`[^musli_value] | **134.58μs** ± 785.59ns | 132.99μs &mdash; 136.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **223.85μs** ± 263.74ns | 223.41μs &mdash; 224.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_wire/report/) |
| `dec/large/postcard` | **89.19μs** ± 94.44ns | 89.03μs &mdash; 89.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/postcard/report/) |
| `dec/large/serde_bincode` | **68.29μs** ± 135.08ns | 68.09μs &mdash; 68.60μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bincode/report/) |
| `dec/large/serde_bitcode` | **101.15μs** ± 199.03ns | 100.82μs &mdash; 101.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bitcode/report/) |
| `dec/large/serde_rmp` | **223.52μs** ± 334.52ns | 222.94μs &mdash; 224.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_rmp/report/) |

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
| `enc/large/musli_descriptive` | **165.24μs** ± 233.54ns | 164.86μs &mdash; 165.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **132.96μs** ± 146.78ns | 132.69μs &mdash; 133.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **32.19μs** ± 53.34ns | 32.11μs &mdash; 32.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_value`[^musli_value] | **733.10μs** ± 1.45μs | 730.98μs &mdash; 736.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **140.11μs** ± 174.82ns | 139.85μs &mdash; 140.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_wire/report/) |
| `enc/large/postcard` | **113.16μs** ± 283.80ns | 112.69μs &mdash; 113.79μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/postcard/report/) |
| `enc/large/serde_bincode` | **42.52μs** ± 55.87ns | 42.43μs &mdash; 42.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bincode/report/) |
| `enc/large/serde_bitcode` | **109.59μs** ± 183.48ns | 109.28μs &mdash; 109.99μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bitcode/report/) |
| `enc/large/serde_rmp` | **155.17μs** ± 162.03ns | 154.88μs &mdash; 155.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_rmp/report/) |


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
| `dec/allocated/musli_descriptive` | **3.42μs** ± 5.36ns | 3.41μs &mdash; 3.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **3.28μs** ± 4.14ns | 3.27μs &mdash; 3.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.41μs** ± 2.36ns | 2.40μs &mdash; 2.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_value`[^musli_value] | **2.13μs** ± 2.35ns | 2.13μs &mdash; 2.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **3.29μs** ± 3.54ns | 3.29μs &mdash; 3.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_wire/report/) |
| `dec/allocated/postcard` | **3.43μs** ± 4.58ns | 3.42μs &mdash; 3.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/postcard/report/) |
| `dec/allocated/serde_bincode` | **3.28μs** ± 3.47ns | 3.28μs &mdash; 3.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bincode/report/) |
| `dec/allocated/serde_bitcode` | **6.13μs** ± 5.22ns | 6.12μs &mdash; 6.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bitcode/report/) |
| `dec/allocated/serde_rmp` | **4.20μs** ± 4.43ns | 4.19μs &mdash; 4.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_rmp/report/) |

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
| `enc/allocated/musli_descriptive` | **719.64ns** ± 0.76ns | 718.25ns &mdash; 721.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **672.26ns** ± 0.86ns | 670.68ns &mdash; 674.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **249.03ns** ± 0.26ns | 248.57ns &mdash; 249.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_value`[^musli_value] | **2.53μs** ± 3.77ns | 2.52μs &mdash; 2.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **646.95ns** ± 0.78ns | 645.52ns &mdash; 648.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_wire/report/) |
| `enc/allocated/postcard` | **1.32μs** ± 1.28ns | 1.31μs &mdash; 1.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/postcard/report/) |
| `enc/allocated/serde_bincode` | **390.27ns** ± 0.41ns | 389.52ns &mdash; 391.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bincode/report/) |
| `enc/allocated/serde_bitcode` | **8.62μs** ± 11.98ns | 8.60μs &mdash; 8.64μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bitcode/report/) |
| `enc/allocated/serde_rmp` | **777.01ns** ± 0.63ns | 775.94ns &mdash; 778.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_rmp/report/) |



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
| `dec/primitives/musli_json` | **4.49μs** ± 6.34ns | 4.48μs &mdash; 4.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **4.38μs** ± 6.54ns | 4.37μs &mdash; 4.39μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/serde_json/report/) |

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
| `enc/primitives/musli_json` | **1.23μs** ± 0.88ns | 1.22μs &mdash; 1.23μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **1.31μs** ± 1.93ns | 1.31μs &mdash; 1.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/serde_json/report/) |


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
| `dec/primpacked/musli_json` | **5.25μs** ± 5.63ns | 5.24μs &mdash; 5.26μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/musli_json/report/) |
| `dec/primpacked/serde_json` | **4.82μs** ± 6.24ns | 4.80μs &mdash; 4.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/serde_json/report/) |

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
| `enc/primpacked/musli_json` | **1.24μs** ± 1.65ns | 1.24μs &mdash; 1.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/musli_json/report/) |
| `enc/primpacked/serde_json` | **1.38μs** ± 1.06ns | 1.38μs &mdash; 1.39μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/serde_json/report/) |


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
| `dec/medium_enum/musli_json` | **9.59μs** ± 10.38ns | 9.58μs &mdash; 9.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/musli_json/report/) |
| `dec/medium_enum/serde_json` | **8.10μs** ± 13.36ns | 8.08μs &mdash; 8.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/serde_json/report/) |

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
| `enc/medium_enum/musli_json` | **2.74μs** ± 3.79ns | 2.74μs &mdash; 2.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/musli_json/report/) |
| `enc/medium_enum/serde_json` | **2.55μs** ± 1.88ns | 2.54μs &mdash; 2.55μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/serde_json/report/) |


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
| `dec/large/musli_json` | **1.15ms** ± 1.59μs | 1.14ms &mdash; 1.15ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **771.78μs** ± 824.55ns | 770.51μs &mdash; 773.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/serde_json/report/) |

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
| `enc/large/musli_json` | **275.49μs** ± 263.66ns | 275.06μs &mdash; 276.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **310.71μs** ± 332.68ns | 310.16μs &mdash; 311.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/serde_json/report/) |


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
| `dec/allocated/musli_json` | **10.71μs** ± 13.01ns | 10.69μs &mdash; 10.74μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **7.70μs** ± 7.57ns | 7.69μs &mdash; 7.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/serde_json/report/) |

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
| `enc/allocated/musli_json` | **2.40μs** ± 2.76ns | 2.40μs &mdash; 2.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **2.45μs** ± 3.40ns | 2.44μs &mdash; 2.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/serde_json/report/) |



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
| `dec/primitives/musli_descriptive` | **556.17ns** ± 0.64ns | 555.09ns &mdash; 557.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **482.20ns** ± 0.49ns | 481.27ns &mdash; 483.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **11.57ns** ± 0.01ns | 11.55ns &mdash; 11.60ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_value`[^musli_value] | **282.57ns** ± 0.33ns | 281.97ns &mdash; 283.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **434.80ns** ± 0.49ns | 433.89ns &mdash; 435.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_wire/report/) |
| `dec/primitives/serde_cbor` | **1.69μs** ± 1.55ns | 1.69μs &mdash; 1.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/serde_cbor/report/) |

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
| `enc/primitives/musli_descriptive` | **504.34ns** ± 0.48ns | 503.44ns &mdash; 505.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **394.62ns** ± 0.53ns | 393.69ns &mdash; 395.75ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **53.12ns** ± 0.06ns | 53.02ns &mdash; 53.24ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.04μs** ± 1.70ns | 1.04μs &mdash; 1.04μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **376.33ns** ± 0.85ns | 374.73ns &mdash; 378.08ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_wire/report/) |
| `enc/primitives/serde_cbor` | **428.49ns** ± 0.30ns | 427.96ns &mdash; 429.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/serde_cbor/report/) |


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
| `dec/primpacked/musli_descriptive` | **572.07ns** ± 0.58ns | 571.17ns &mdash; 573.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **507.27ns** ± 0.54ns | 506.28ns &mdash; 508.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **22.18ns** ± 0.02ns | 22.13ns &mdash; 22.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **285.22ns** ± 0.24ns | 284.80ns &mdash; 285.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **478.54ns** ± 0.41ns | 477.80ns &mdash; 479.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/serde_cbor` | **1.81μs** ± 3.34ns | 1.80μs &mdash; 1.81μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/serde_cbor/report/) |

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
| `enc/primpacked/musli_descriptive` | **459.74ns** ± 0.51ns | 458.76ns &mdash; 460.75ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **338.64ns** ± 0.59ns | 337.51ns &mdash; 339.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **57.22ns** ± 0.05ns | 57.13ns &mdash; 57.33ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **1.20μs** ± 1.62ns | 1.19μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **337.93ns** ± 0.38ns | 337.22ns &mdash; 338.71ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/serde_cbor` | **488.22ns** ± 0.51ns | 487.30ns &mdash; 489.29ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/serde_cbor/report/) |


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
| `dec/medium_enum/musli_descriptive` | **1.48μs** ± 2.66ns | 1.47μs &mdash; 1.48μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.26μs** ± 1.16ns | 1.26μs &mdash; 1.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **467.89ns** ± 0.63ns | 466.80ns &mdash; 469.24ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **793.88ns** ± 0.80ns | 792.63ns &mdash; 795.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **1.27μs** ± 1.36ns | 1.27μs &mdash; 1.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/serde_cbor` | **4.71μs** ± 5.14ns | 4.70μs &mdash; 4.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/serde_cbor/report/) |

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
| `enc/medium_enum/musli_descriptive` | **1.17μs** ± 1.62ns | 1.17μs &mdash; 1.17μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **831.65ns** ± 0.71ns | 830.40ns &mdash; 833.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **274.81ns** ± 0.23ns | 274.39ns &mdash; 275.29ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **3.05μs** ± 2.49ns | 3.04μs &mdash; 3.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **877.66ns** ± 0.96ns | 876.01ns &mdash; 879.75ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/serde_cbor` | **1.03μs** ± 0.92ns | 1.03μs &mdash; 1.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/serde_cbor/report/) |


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
| `dec/large/musli_descriptive` | **252.05μs** ± 382.91ns | 251.40μs &mdash; 252.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **201.21μs** ± 213.20ns | 200.84μs &mdash; 201.67μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **43.15μs** ± 37.90ns | 43.08μs &mdash; 43.23μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_value`[^musli_value] | **129.12μs** ± 265.71ns | 128.64μs &mdash; 129.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **227.62μs** ± 268.40ns | 227.17μs &mdash; 228.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_wire/report/) |
| `dec/large/serde_cbor` | **572.42μs** ± 560.26ns | 571.43μs &mdash; 573.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/serde_cbor/report/) |

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
| `enc/large/musli_descriptive` | **187.21μs** ± 154.31ns | 186.95μs &mdash; 187.55μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **144.98μs** ± 137.66ns | 144.74μs &mdash; 145.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **33.94μs** ± 27.59ns | 33.89μs &mdash; 34.00μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_value`[^musli_value] | **768.44μs** ± 1.13μs | 766.50μs &mdash; 770.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **155.09μs** ± 146.82ns | 154.83μs &mdash; 155.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_wire/report/) |
| `enc/large/serde_cbor` | **172.07μs** ± 248.19ns | 171.66μs &mdash; 172.63μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/serde_cbor/report/) |


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
| `dec/allocated/musli_descriptive` | **2.49μs** ± 2.64ns | 2.48μs &mdash; 2.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **2.26μs** ± 2.34ns | 2.26μs &mdash; 2.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **1.75μs** ± 2.66ns | 1.75μs &mdash; 1.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_value`[^musli_value] | **1.46μs** ± 1.76ns | 1.46μs &mdash; 1.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **2.38μs** ± 4.21ns | 2.37μs &mdash; 2.39μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_wire/report/) |
| `dec/allocated/serde_cbor` | **5.08μs** ± 5.04ns | 5.07μs &mdash; 5.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/serde_cbor/report/) |

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
| `enc/allocated/musli_descriptive` | **540.83ns** ± 0.72ns | 539.51ns &mdash; 542.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **482.25ns** ± 0.49ns | 481.36ns &mdash; 483.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **212.04ns** ± 0.22ns | 211.62ns &mdash; 212.50ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_value`[^musli_value] | **1.96μs** ± 2.54ns | 1.95μs &mdash; 1.96μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **439.35ns** ± 0.40ns | 438.62ns &mdash; 440.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_wire/report/) |
| `enc/allocated/serde_cbor` | **656.73ns** ± 0.71ns | 655.63ns &mdash; 658.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/serde_cbor/report/) |



### Speedy

> **Missing features:**
> - `isize` - `isize` types are not supported.
> - `cstring` - `CString`'s are not supported.

This is a test suite for speedy features.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/report/)
* [Sizes](#speedy-sizes)

<table>
<tr>
<th colspan="3">
<code>speedy/dec/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primitives/musli_descriptive` | **645.23ns** ± 0.54ns | 644.24ns &mdash; 646.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **567.66ns** ± 0.62ns | 566.61ns &mdash; 569.00ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **15.97ns** ± 0.02ns | 15.94ns &mdash; 16.01ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_value`[^musli_value] | **291.99ns** ± 0.41ns | 291.37ns &mdash; 292.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **544.75ns** ± 0.89ns | 543.20ns &mdash; 546.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_wire/report/) |
| `dec/primitives/speedy` | **17.79ns** ± 0.02ns | 17.75ns &mdash; 17.84ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/speedy/report/) |

<table>
<tr>
<th colspan="3">
<code>speedy/enc/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primitives/musli_descriptive` | **811.75ns** ± 1.04ns | 810.01ns &mdash; 814.06ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **704.97ns** ± 0.64ns | 703.87ns &mdash; 706.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **59.22ns** ± 0.08ns | 59.10ns &mdash; 59.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.06μs** ± 1.13ns | 1.06μs &mdash; 1.06μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **659.38ns** ± 0.76ns | 658.04ns &mdash; 661.01ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_wire/report/) |
| `enc/primitives/speedy` | **16.16ns** ± 0.01ns | 16.13ns &mdash; 16.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/speedy/report/) |


<table>
<tr>
<th colspan="3">
<code>speedy/dec/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primpacked/musli_descriptive` | **703.62ns** ± 0.72ns | 702.32ns &mdash; 705.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **635.76ns** ± 0.68ns | 634.52ns &mdash; 637.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **23.59ns** ± 0.02ns | 23.55ns &mdash; 23.62ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **303.91ns** ± 0.29ns | 303.39ns &mdash; 304.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **573.08ns** ± 0.51ns | 572.16ns &mdash; 574.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/speedy` | **20.82ns** ± 0.03ns | 20.76ns &mdash; 20.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/speedy/report/) |

<table>
<tr>
<th colspan="3">
<code>speedy/enc/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primpacked/musli_descriptive` | **772.31ns** ± 0.99ns | 770.49ns &mdash; 774.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **650.85ns** ± 0.76ns | 649.42ns &mdash; 652.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **62.85ns** ± 0.07ns | 62.72ns &mdash; 63.00ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **1.22μs** ± 1.47ns | 1.21μs &mdash; 1.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **652.02ns** ± 0.69ns | 650.73ns &mdash; 653.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/speedy` | **16.84ns** ± 0.02ns | 16.80ns &mdash; 16.88ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/speedy/report/) |


<table>
<tr>
<th colspan="3">
<code>speedy/dec/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/medium_enum/musli_descriptive` | **1.59μs** ± 1.69ns | 1.58μs &mdash; 1.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.46μs** ± 1.40ns | 1.45μs &mdash; 1.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **547.54ns** ± 0.41ns | 546.76ns &mdash; 548.38ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **792.47ns** ± 1.14ns | 790.49ns &mdash; 794.94ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **1.36μs** ± 2.50ns | 1.36μs &mdash; 1.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/speedy` | **584.92ns** ± 0.67ns | 583.72ns &mdash; 586.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/speedy/report/) |

<table>
<tr>
<th colspan="3">
<code>speedy/enc/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/medium_enum/musli_descriptive` | **1.41μs** ± 1.71ns | 1.41μs &mdash; 1.42μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **1.16μs** ± 1.32ns | 1.15μs &mdash; 1.16μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **260.63ns** ± 0.19ns | 260.29ns &mdash; 261.04ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **3.01μs** ± 3.75ns | 3.00μs &mdash; 3.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **1.13μs** ± 1.00ns | 1.13μs &mdash; 1.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/speedy` | **306.29ns** ± 0.34ns | 305.63ns &mdash; 306.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/speedy/report/) |


<table>
<tr>
<th colspan="3">
<code>speedy/dec/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/large/musli_descriptive` | **275.14μs** ± 269.03ns | 274.70μs &mdash; 275.74μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **221.42μs** ± 224.02ns | 221.04μs &mdash; 221.91μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **77.15μs** ± 103.65ns | 76.97μs &mdash; 77.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_value`[^musli_value] | **140.34μs** ± 524.34ns | 139.43μs &mdash; 141.48μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **256.28μs** ± 305.00ns | 255.81μs &mdash; 256.98μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_wire/report/) |
| `dec/large/speedy` | **71.17μs** ± 81.37ns | 71.03μs &mdash; 71.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/speedy/report/) |

<table>
<tr>
<th colspan="3">
<code>speedy/enc/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/large/musli_descriptive` | **178.12μs** ± 173.96ns | 177.82μs &mdash; 178.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **145.74μs** ± 164.49ns | 145.46μs &mdash; 146.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **34.03μs** ± 35.37ns | 33.97μs &mdash; 34.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_value`[^musli_value] | **733.26μs** ± 2.38μs | 729.08μs &mdash; 738.38μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **152.38μs** ± 144.34ns | 152.14μs &mdash; 152.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_wire/report/) |
| `enc/large/speedy` | **20.39μs** ± 21.12ns | 20.35μs &mdash; 20.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/speedy/report/) |


<table>
<tr>
<th colspan="3">
<code>speedy/dec/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/allocated/musli_descriptive` | **3.80μs** ± 5.79ns | 3.79μs &mdash; 3.81μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **3.36μs** ± 3.76ns | 3.35μs &mdash; 3.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.79μs** ± 2.99ns | 2.78μs &mdash; 2.79μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_value`[^musli_value] | **1.94μs** ± 2.47ns | 1.94μs &mdash; 1.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **3.60μs** ± 4.04ns | 3.59μs &mdash; 3.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_wire/report/) |
| `dec/allocated/speedy` | **3.36μs** ± 4.75ns | 3.35μs &mdash; 3.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/speedy/report/) |

<table>
<tr>
<th colspan="3">
<code>speedy/enc/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/allocated/musli_descriptive` | **723.32ns** ± 1.39ns | 721.10ns &mdash; 726.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **653.26ns** ± 0.92ns | 651.63ns &mdash; 655.24ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **239.87ns** ± 0.39ns | 239.15ns &mdash; 240.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_value`[^musli_value] | **2.32μs** ± 3.05ns | 2.31μs &mdash; 2.33μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **644.96ns** ± 0.64ns | 643.73ns &mdash; 646.25ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_wire/report/) |
| `enc/allocated/speedy` | **503.87ns** ± 0.59ns | 502.74ns &mdash; 505.05ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/speedy/report/) |



### Müsli vs rkyv

> **Missing features:**
> - `cstring` - `CString`'s are not supported.
> - `map` - Maps like `MashMap<K, V>` are not supported.
> - `set` - Sets like `HashSet<T>` are not supported.
> - `tuple` - Tuples like `(u32, u32)` are not supported.
> - `usize` - `usize` types are not supported.
> - `isize` - `isize` types are not supported.

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
| `dec/primitives/musli_zerocopy` | **4.00ns** ± 0.00ns | 4.00ns &mdash; 4.01ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primitives/musli_zerocopy/report/) |
| `dec/primitives/rkyv` | **14.62ns** ± 0.02ns | 14.59ns &mdash; 14.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primitives/rkyv/report/) |

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
| `enc/primitives/musli_zerocopy` | **20.08ns** ± 0.03ns | 20.04ns &mdash; 20.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primitives/musli_zerocopy/report/) |
| `enc/primitives/rkyv` | **32.57ns** ± 0.03ns | 32.51ns &mdash; 32.64ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primitives/rkyv/report/) |


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
| `dec/primpacked/musli_zerocopy` | **2.65ns** ± 0.00ns | 2.65ns &mdash; 2.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primpacked/musli_zerocopy/report/) |
| `dec/primpacked/rkyv` | **14.20ns** ± 0.01ns | 14.18ns &mdash; 14.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/dec_primpacked/rkyv/report/) |

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
| `enc/primpacked/musli_zerocopy` | **16.81ns** ± 0.01ns | 16.78ns &mdash; 16.84ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primpacked/musli_zerocopy/report/) |
| `enc/primpacked/rkyv` | **33.24ns** ± 0.05ns | 33.17ns &mdash; 33.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/enc_primpacked/rkyv/report/) |



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
| `dec/primpacked/musli_zerocopy` | **2.66ns** ± 0.00ns | 2.65ns &mdash; 2.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/musli_zerocopy/report/) |
| `dec/primpacked/zerocopy` | **6.64ns** ± 0.01ns | 6.63ns &mdash; 6.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/zerocopy/report/) |

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
| `enc/primpacked/musli_zerocopy` | **17.85ns** ± 0.02ns | 17.83ns &mdash; 17.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/musli_zerocopy/report/) |
| `enc/primpacked/zerocopy` | **8.42ns** ± 0.01ns | 8.40ns &mdash; 8.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/zerocopy/report/) |



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
| `dec/primitives/derive_bitcode` | **251.71ns** ± 0.37ns | 251.07ns &mdash; 252.50ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/derive_bitcode/report/) |
| `dec/primitives/musli_descriptive` | **719.42ns** ± 0.62ns | 718.26ns &mdash; 720.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **637.39ns** ± 0.83ns | 635.87ns &mdash; 639.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **13.36ns** ± 0.01ns | 13.34ns &mdash; 13.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_wire` | **578.82ns** ± 0.49ns | 577.94ns &mdash; 579.85ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/derive_bitcode` | **1.30μs** ± 1.67ns | 1.30μs &mdash; 1.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/derive_bitcode/report/) |
| `enc/primitives/musli_descriptive` | **876.29ns** ± 1.06ns | 874.31ns &mdash; 878.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **769.70ns** ± 0.82ns | 768.28ns &mdash; 771.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **61.91ns** ± 0.04ns | 61.84ns &mdash; 62.00ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_wire` | **696.10ns** ± 0.76ns | 694.74ns &mdash; 697.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_wire/report/) |


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
| `dec/primpacked/derive_bitcode` | **250.14ns** ± 0.24ns | 249.71ns &mdash; 250.64ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/derive_bitcode/report/) |
| `dec/primpacked/musli_descriptive` | **726.97ns** ± 0.84ns | 725.50ns &mdash; 728.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **671.99ns** ± 1.20ns | 669.82ns &mdash; 674.51ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **26.11ns** ± 0.02ns | 26.07ns &mdash; 26.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_wire` | **576.13ns** ± 0.69ns | 575.01ns &mdash; 577.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_wire/report/) |

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
| `enc/primpacked/derive_bitcode` | **1.31μs** ± 1.38ns | 1.30μs &mdash; 1.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/derive_bitcode/report/) |
| `enc/primpacked/musli_descriptive` | **781.25ns** ± 0.81ns | 779.74ns &mdash; 782.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **666.80ns** ± 1.11ns | 664.71ns &mdash; 669.08ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **66.07ns** ± 0.08ns | 65.93ns &mdash; 66.24ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_wire` | **652.96ns** ± 0.61ns | 651.81ns &mdash; 654.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_wire/report/) |


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
| `dec/medium_enum/derive_bitcode` | **3.27μs** ± 3.68ns | 3.27μs &mdash; 3.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/derive_bitcode/report/) |
| `dec/medium_enum/musli_descriptive` | **1.83μs** ± 2.37ns | 1.82μs &mdash; 1.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **1.62μs** ± 1.60ns | 1.62μs &mdash; 1.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **697.13ns** ± 1.09ns | 695.35ns &mdash; 699.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_wire` | **1.56μs** ± 1.75ns | 1.55μs &mdash; 1.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_wire/report/) |

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
| `enc/medium_enum/derive_bitcode` | **13.59μs** ± 15.90ns | 13.56μs &mdash; 13.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/derive_bitcode/report/) |
| `enc/medium_enum/musli_descriptive` | **1.49μs** ± 1.50ns | 1.48μs &mdash; 1.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **1.20μs** ± 1.76ns | 1.19μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **267.76ns** ± 0.34ns | 267.23ns &mdash; 268.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_wire` | **1.20μs** ± 1.12ns | 1.20μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_wire/report/) |


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
| `dec/large/derive_bitcode` | **33.22μs** ± 47.17ns | 33.13μs &mdash; 33.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/derive_bitcode/report/) |
| `dec/large/musli_descriptive` | **245.52μs** ± 317.73ns | 245.03μs &mdash; 246.24μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **196.62μs** ± 162.48ns | 196.33μs &mdash; 196.96μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **55.60μs** ± 49.59ns | 55.51μs &mdash; 55.71μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_wire` | **228.42μs** ± 273.79ns | 227.92μs &mdash; 228.99μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_wire/report/) |

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
| `enc/large/derive_bitcode` | **86.17μs** ± 203.41ns | 85.83μs &mdash; 86.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/derive_bitcode/report/) |
| `enc/large/musli_descriptive` | **171.23μs** ± 238.98ns | 170.82μs &mdash; 171.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **135.60μs** ± 183.85ns | 135.34μs &mdash; 136.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **32.24μs** ± 79.88ns | 32.11μs &mdash; 32.42μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_wire` | **144.95μs** ± 203.81ns | 144.61μs &mdash; 145.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_wire/report/) |


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
| `dec/allocated/derive_bitcode` | **3.93μs** ± 5.10ns | 3.92μs &mdash; 3.94μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/derive_bitcode/report/) |
| `dec/allocated/musli_descriptive` | **3.70μs** ± 3.76ns | 3.69μs &mdash; 3.71μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **3.48μs** ± 3.74ns | 3.47μs &mdash; 3.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.84μs** ± 2.69ns | 2.83μs &mdash; 2.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_wire` | **3.65μs** ± 3.01ns | 3.65μs &mdash; 3.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_wire/report/) |

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
| `enc/allocated/derive_bitcode` | **7.28μs** ± 11.43ns | 7.26μs &mdash; 7.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/derive_bitcode/report/) |
| `enc/allocated/musli_descriptive` | **698.53ns** ± 0.52ns | 697.61ns &mdash; 699.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **763.17ns** ± 0.60ns | 762.21ns &mdash; 764.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **243.30ns** ± 0.32ns | 242.71ns &mdash; 243.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_wire` | **636.03ns** ± 0.99ns | 634.20ns &mdash; 638.07ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_wire/report/) |



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
| `dec/primitives/bson`[^bson] | **2.99μs** ± 3.00ns | 2.99μs &mdash; 3.00μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/bson/report/) |
| `dec/primitives/musli_descriptive` | **542.67ns** ± 0.51ns | 541.76ns &mdash; 543.77ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_storage` | **464.96ns** ± 0.55ns | 463.94ns &mdash; 466.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_storage_packed` | **11.51ns** ± 0.01ns | 11.49ns &mdash; 11.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_storage_packed/report/) |
| `dec/primitives/musli_wire` | **423.77ns** ± 0.50ns | 422.92ns &mdash; 424.88ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/bson`[^bson] | **1.35μs** ± 1.17ns | 1.34μs &mdash; 1.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/bson/report/) |
| `enc/primitives/musli_descriptive` | **488.60ns** ± 0.61ns | 487.51ns &mdash; 489.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_storage` | **380.33ns** ± 0.30ns | 379.79ns &mdash; 380.97ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_storage_packed` | **53.04ns** ± 0.04ns | 52.97ns &mdash; 53.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_storage_packed/report/) |
| `enc/primitives/musli_wire` | **355.11ns** ± 0.53ns | 354.10ns &mdash; 356.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_wire/report/) |


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
| `dec/primpacked/bson`[^bson] | **3.87μs** ± 5.67ns | 3.86μs &mdash; 3.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/bson/report/) |
| `dec/primpacked/musli_descriptive` | **571.88ns** ± 0.54ns | 570.91ns &mdash; 573.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_storage` | **502.21ns** ± 0.65ns | 501.04ns &mdash; 503.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_storage_packed` | **22.17ns** ± 0.01ns | 22.14ns &mdash; 22.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_storage_packed/report/) |
| `dec/primpacked/musli_wire` | **485.25ns** ± 0.52ns | 484.33ns &mdash; 486.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_wire/report/) |

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
| `enc/primpacked/bson`[^bson] | **2.47μs** ± 3.75ns | 2.47μs &mdash; 2.48μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/bson/report/) |
| `enc/primpacked/musli_descriptive` | **463.05ns** ± 0.65ns | 461.82ns &mdash; 464.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_storage` | **340.03ns** ± 0.68ns | 338.72ns &mdash; 341.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_storage_packed` | **57.46ns** ± 0.06ns | 57.35ns &mdash; 57.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_storage_packed/report/) |
| `enc/primpacked/musli_wire` | **331.83ns** ± 0.45ns | 331.06ns &mdash; 332.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_wire/report/) |


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
| `dec/medium_enum/bson`[^bson] | **8.01μs** ± 7.51ns | 8.00μs &mdash; 8.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/bson/report/) |
| `dec/medium_enum/musli_descriptive` | **1.17μs** ± 1.66ns | 1.16μs &mdash; 1.17μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_storage` | **988.83ns** ± 1.67ns | 985.83ns &mdash; 992.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_storage_packed` | **313.01ns** ± 0.33ns | 312.41ns &mdash; 313.71ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_storage_packed/report/) |
| `dec/medium_enum/musli_wire` | **963.99ns** ± 1.04ns | 962.17ns &mdash; 966.24ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_wire/report/) |

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
| `enc/medium_enum/bson`[^bson] | **5.35μs** ± 13.13ns | 5.33μs &mdash; 5.38μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/bson/report/) |
| `enc/medium_enum/musli_descriptive` | **946.58ns** ± 0.92ns | 944.88ns &mdash; 948.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_storage` | **715.58ns** ± 0.94ns | 713.81ns &mdash; 717.51ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_storage_packed` | **205.16ns** ± 0.17ns | 204.86ns &mdash; 205.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_storage_packed/report/) |
| `enc/medium_enum/musli_wire` | **715.62ns** ± 0.73ns | 714.30ns &mdash; 717.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_wire/report/) |


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
| `dec/large/bson`[^bson] | **1.77ms** ± 941.03ns | 1.77ms &mdash; 1.77ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/bson/report/) |
| `dec/large/musli_descriptive` | **311.75μs** ± 279.76ns | 311.26μs &mdash; 312.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_descriptive/report/) |
| `dec/large/musli_storage` | **253.46μs** ± 262.60ns | 252.99μs &mdash; 254.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_storage/report/) |
| `dec/large/musli_storage_packed` | **86.13μs** ± 109.72ns | 85.94μs &mdash; 86.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_storage_packed/report/) |
| `dec/large/musli_wire` | **293.71μs** ± 250.58ns | 293.25μs &mdash; 294.23μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_wire/report/) |

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
| `enc/large/bson`[^bson] | **964.91μs** ± 1.34μs | 962.60μs &mdash; 967.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/bson/report/) |
| `enc/large/musli_descriptive` | **206.30μs** ± 243.15ns | 205.94μs &mdash; 206.86μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_descriptive/report/) |
| `enc/large/musli_storage` | **159.45μs** ± 189.18ns | 159.11μs &mdash; 159.85μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_storage/report/) |
| `enc/large/musli_storage_packed` | **40.50μs** ± 53.73ns | 40.42μs &mdash; 40.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_storage_packed/report/) |
| `enc/large/musli_wire` | **168.04μs** ± 165.88ns | 167.77μs &mdash; 168.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_wire/report/) |


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
| `dec/allocated/bson`[^bson] | **7.68μs** ± 9.38ns | 7.66μs &mdash; 7.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/bson/report/) |
| `dec/allocated/musli_descriptive` | **3.12μs** ± 3.27ns | 3.11μs &mdash; 3.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_storage` | **2.91μs** ± 3.37ns | 2.91μs &mdash; 2.92μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_storage_packed` | **2.49μs** ± 2.74ns | 2.49μs &mdash; 2.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_storage_packed/report/) |
| `dec/allocated/musli_wire` | **2.99μs** ± 2.70ns | 2.99μs &mdash; 3.00μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_wire/report/) |

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
| `enc/allocated/bson`[^bson] | **2.64μs** ± 4.87ns | 2.63μs &mdash; 2.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/bson/report/) |
| `enc/allocated/musli_descriptive` | **466.24ns** ± 0.56ns | 465.24ns &mdash; 467.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_storage` | **416.24ns** ± 0.39ns | 415.54ns &mdash; 417.07ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_storage_packed` | **202.33ns** ± 0.23ns | 201.92ns &mdash; 202.84ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_storage_packed/report/) |
| `enc/allocated/musli_wire` | **377.35ns** ± 0.50ns | 376.37ns &mdash; 378.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_wire/report/) |



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
| `dec/primitives/miniserde` | **2.11μs** ± 2.13ns | 2.11μs &mdash; 2.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/miniserde/report/) |
| `dec/primitives/musli_json` | **2.80μs** ± 2.77ns | 2.79μs &mdash; 2.80μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **2.20μs** ± 2.16ns | 2.20μs &mdash; 2.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/serde_json/report/) |

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
| `enc/primitives/miniserde` | **2.49μs** ± 3.15ns | 2.48μs &mdash; 2.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/miniserde/report/) |
| `enc/primitives/musli_json` | **808.11ns** ± 1.23ns | 805.86ns &mdash; 810.68ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **958.47ns** ± 1.18ns | 956.34ns &mdash; 960.95ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/serde_json/report/) |


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
| `dec/primpacked/miniserde` | **2.84μs** ± 2.41ns | 2.84μs &mdash; 2.85μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/miniserde/report/) |
| `dec/primpacked/musli_json` | **3.96μs** ± 3.33ns | 3.95μs &mdash; 3.97μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/musli_json/report/) |
| `dec/primpacked/serde_json` | **2.83μs** ± 3.10ns | 2.82μs &mdash; 2.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/serde_json/report/) |

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
| `enc/primpacked/miniserde` | **2.97μs** ± 2.37ns | 2.97μs &mdash; 2.98μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/miniserde/report/) |
| `enc/primpacked/musli_json` | **932.31ns** ± 0.91ns | 930.73ns &mdash; 934.29ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/musli_json/report/) |
| `enc/primpacked/serde_json` | **1.13μs** ± 1.12ns | 1.13μs &mdash; 1.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/serde_json/report/) |


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
| `dec/medium_enum/miniserde` | **67.84ns** ± 0.11ns | 67.65ns &mdash; 68.08ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/miniserde/report/) |
| `dec/medium_enum/musli_json` | **54.99ns** ± 0.05ns | 54.91ns &mdash; 55.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/musli_json/report/) |
| `dec/medium_enum/serde_json` | **73.91ns** ± 0.06ns | 73.80ns &mdash; 74.04ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/serde_json/report/) |

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
| `enc/medium_enum/miniserde` | **97.14ns** ± 0.10ns | 96.96ns &mdash; 97.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/miniserde/report/) |
| `enc/medium_enum/musli_json` | **24.80ns** ± 0.02ns | 24.76ns &mdash; 24.85ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/musli_json/report/) |
| `enc/medium_enum/serde_json` | **28.87ns** ± 0.02ns | 28.84ns &mdash; 28.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/serde_json/report/) |


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
| `dec/large/miniserde` | **188.14μs** ± 191.41ns | 187.79μs &mdash; 188.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/miniserde/report/) |
| `dec/large/musli_json` | **276.46μs** ± 260.96ns | 276.02μs &mdash; 277.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **219.08μs** ± 322.68ns | 218.55μs &mdash; 219.79μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/serde_json/report/) |

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
| `enc/large/miniserde` | **150.20μs** ± 123.58ns | 149.99μs &mdash; 150.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/miniserde/report/) |
| `enc/large/musli_json` | **78.67μs** ± 78.10ns | 78.54μs &mdash; 78.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **93.82μs** ± 153.01ns | 93.55μs &mdash; 94.15μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/serde_json/report/) |


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
| `dec/allocated/miniserde` | **571.01ns** ± 0.72ns | 569.73ns &mdash; 572.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/miniserde/report/) |
| `dec/allocated/musli_json` | **543.83ns** ± 0.63ns | 542.67ns &mdash; 545.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **400.35ns** ± 0.34ns | 399.73ns &mdash; 401.07ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/serde_json/report/) |

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
| `enc/allocated/miniserde` | **661.84ns** ± 0.85ns | 660.32ns &mdash; 663.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/miniserde/report/) |
| `enc/allocated/musli_json` | **135.60ns** ± 0.10ns | 135.41ns &mdash; 135.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **150.63ns** ± 0.19ns | 150.31ns &mdash; 151.05ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/serde_json/report/) |



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
| `musli_storage_packed` | <a title="samples: 500, min: 95, max: 95, stddev: 0">95.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 21949, max: 68407, stddev: 11697.302535627605">49343.70 ± 11697.30</a> | <a title="samples: 100, min: 417, max: 1010, stddev: 117.76399237457943">711.89 ± 117.76</a> | <a title="samples: 4000, min: 16, max: 191, stddev: 53.975848154521344">59.40 ± 53.98</a> |
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
| `musli_storage_packed` | <a title="samples: 500, min: 63, max: 63, stddev: 0">63.00 ± 0.00</a> | <a title="samples: 500, min: 64, max: 64, stddev: 0">64.00 ± 0.00</a> | <a title="samples: 10, min: 17885, max: 49589, stddev: 9851.19105946078">30538.90 ± 9851.19</a> | <a title="samples: 100, min: 350, max: 799, stddev: 100.1102012783912">542.74 ± 100.11</a> | <a title="samples: 4000, min: 16, max: 190, stddev: 47.92615269284045">55.32 ± 47.93</a> |
| `musli_wire` | <a title="samples: 500, min: 96, max: 106, stddev: 1.7524143345681649">101.86 ± 1.75</a> | <a title="samples: 500, min: 102, max: 111, stddev: 1.7655310815729104">106.83 ± 1.77</a> | <a title="samples: 10, min: 16463, max: 43841, stddev: 8662.557910917536">28513.80 ± 8662.56</a> | <a title="samples: 100, min: 288, max: 726, stddev: 98.035046794501">477.64 ± 98.04</a> | <a title="samples: 4000, min: 3, max: 173, stddev: 49.88740967418499">44.07 ± 49.89</a> |
| `serde_cbor`[^i128] | <a title="samples: 500, min: 210, max: 213, stddev: 0.5346961754117986">212.69 ± 0.53</a> | <a title="samples: 500, min: 218, max: 222, stddev: 0.847610759724064">221.17 ± 0.85</a> | <a title="samples: 10, min: 20033, max: 47027, stddev: 9429.151033364564">34759.30 ± 9429.15</a> | <a title="samples: 100, min: 380, max: 815, stddev: 97.29138656633484">566.69 ± 97.29</a> | <a title="samples: 4000, min: 6, max: 251, stddev: 80.46084400152334">65.78 ± 80.46</a> |

#### Speedy sizes

> **Missing features:**
> - `isize` - `isize` types are not supported.
> - `cstring` - `CString`'s are not supported.

This is a test suite for speedy features.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` |
| - | - | - | - | - | - |
| `musli_descriptive` | <a title="samples: 500, min: 142, max: 151, stddev: 1.5066187308008552">147.31 ± 1.51</a> | <a title="samples: 500, min: 148, max: 157, stddev: 1.4568459081179361">153.36 ± 1.46</a> | <a title="samples: 10, min: 18658, max: 61872, stddev: 15493.732655819254">41718.30 ± 15493.73</a> | <a title="samples: 100, min: 321, max: 882, stddev: 119.63710920947565">620.89 ± 119.64</a> | <a title="samples: 4000, min: 4, max: 179, stddev: 61.69536744713939">52.25 ± 61.70</a> |
| `musli_storage` | <a title="samples: 500, min: 113, max: 120, stddev: 1.3242356285797454">117.32 ± 1.32</a> | <a title="samples: 500, min: 115, max: 123, stddev: 1.2658135723715367">120.35 ± 1.27</a> | <a title="samples: 10, min: 15373, max: 49935, stddev: 12489.609979498959">34094.60 ± 12489.61</a> | <a title="samples: 100, min: 295, max: 849, stddev: 118.15400289452744">592.46 ± 118.15</a> | <a title="samples: 4000, min: 2, max: 146, stddev: 50.33655391400514">42.07 ± 50.34</a> |
| `musli_storage_packed` | <a title="samples: 500, min: 87, max: 87, stddev: 0">87.00 ± 0.00</a> | <a title="samples: 500, min: 88, max: 88, stddev: 0">88.00 ± 0.00</a> | <a title="samples: 10, min: 18035, max: 61948, stddev: 16869.39717950822">41814.00 ± 16869.40</a> | <a title="samples: 100, min: 373, max: 944, stddev: 122.85355957399035">678.73 ± 122.85</a> | <a title="samples: 4000, min: 16, max: 188, stddev: 52.300722006847316">58.41 ± 52.30</a> |
| `musli_wire` | <a title="samples: 500, min: 126, max: 136, stddev: 1.8188908708330995">131.81 ± 1.82</a> | <a title="samples: 500, min: 131, max: 141, stddev: 1.6698970028118476">136.96 ± 1.67</a> | <a title="samples: 10, min: 17410, max: 57501, stddev: 14496.84311013953">38859.80 ± 14496.84</a> | <a title="samples: 100, min: 308, max: 871, stddev: 119.77968567332277">608.63 ± 119.78</a> | <a title="samples: 4000, min: 3, max: 171, stddev: 56.554816770435096">47.90 ± 56.55</a> |
| `speedy` | <a title="samples: 500, min: 87, max: 87, stddev: 0">87.00 ± 0.00</a> | <a title="samples: 500, min: 88, max: 88, stddev: 0">88.00 ± 0.00</a> | <a title="samples: 10, min: 12551, max: 38232, stddev: 9463.757110154507">27314.40 ± 9463.76</a> | <a title="samples: 100, min: 313, max: 872, stddev: 119.94969737352403">614.49 ± 119.95</a> | <a title="samples: 4000, min: 4, max: 152, stddev: 44.03843517244322">39.91 ± 44.04</a> |

#### Müsli vs rkyv sizes

> **Missing features:**
> - `cstring` - `CString`'s are not supported.
> - `map` - Maps like `MashMap<K, V>` are not supported.
> - `set` - Sets like `HashSet<T>` are not supported.
> - `tuple` - Tuples like `(u32, u32)` are not supported.
> - `usize` - `usize` types are not supported.
> - `isize` - `isize` types are not supported.

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
| `musli_storage_packed` | <a title="samples: 500, min: 95, max: 95, stddev: 0">95.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 21949, max: 68407, stddev: 11697.302535627605">49343.70 ± 11697.30</a> | <a title="samples: 100, min: 428, max: 987, stddev: 113.04594243050035">698.43 ± 113.05</a> | <a title="samples: 4000, min: 16, max: 191, stddev: 53.936636221935856">59.32 ± 53.94</a> |
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
| `musli_storage_packed` | <a title="samples: 500, min: 63, max: 63, stddev: 0">63.00 ± 0.00</a> | <a title="samples: 500, min: 64, max: 64, stddev: 0">64.00 ± 0.00</a> | <a title="samples: 10, min: 25131, max: 68436, stddev: 14716.659497317998">48084.20 ± 14716.66</a> | <a title="samples: 100, min: 426, max: 903, stddev: 105.7507654818631">659.34 ± 105.75</a> | <a title="samples: 2500, min: 16, max: 191, stddev: 50.73109837407374">65.01 ± 50.73</a> |
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
