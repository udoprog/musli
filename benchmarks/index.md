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
- `mesh` - A mesh containing triangles.

The following are one section for each kind of benchmark we perform. They range from "Full features" to more specialized ones like zerocopy comparisons.
- [**Full features**](#full-features) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/report/), [Sizes](#full-features-sizes))
- [**Text-based formats**](#text-based-formats) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/report/), [Sizes](#text-based-formats-sizes))
- [**Fewer features**](#fewer-features) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/report/), [Sizes](#fewer-features-sizes))
- [**Speedy**](#speedy) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/report/), [Sizes](#speedy-sizes))
- [**ε-serde**](#ε-serde) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/report/), [Sizes](#ε-serde-sizes))
- [**Müsli vs zerocopy**](#müsli-vs-zerocopy) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/report/), [Sizes](#müsli-vs-zerocopy-sizes))
- [**Bitcode derive**](#bitcode-derive) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/report/), [Sizes](#bitcode-derive-sizes))
- [**BSON**](#bson) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/report/), [Sizes](#bson-sizes))
- [**Miniserde**](#miniserde) ([Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/report/), [Sizes](#miniserde-sizes))

Below you'll also find [size comparisons](#size-comparisons).

## System Information

CPU: Intel(R) Core(TM) i9-9900K CPU @ 3.60GHz 4797MHz
Memory: 67319MB

## Reports

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
| `dec/primitives/musli_descriptive` | **374.70ns** ± 0.15ns | 374.50ns &mdash; 374.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **6.82ns** ± 0.04ns | 6.77ns &mdash; 6.88ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **217.85ns** ± 0.38ns | 217.30ns &mdash; 218.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_value`[^musli_value] | **78.96ns** ± 0.12ns | 78.80ns &mdash; 79.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **229.44ns** ± 1.48ns | 227.34ns &mdash; 231.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_wire/report/) |
| `dec/primitives/postcard` | **100.18ns** ± 0.10ns | 100.03ns &mdash; 100.33ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/postcard/report/) |
| `dec/primitives/serde_bincode` | **38.60ns** ± 0.25ns | 38.25ns &mdash; 38.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bincode/report/) |
| `dec/primitives/serde_bitcode` | **366.22ns** ± 0.06ns | 366.13ns &mdash; 366.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bitcode/report/) |
| `dec/primitives/serde_rmp` | **113.42ns** ± 0.32ns | 112.97ns &mdash; 113.86ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_rmp/report/) |

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
| `enc/primitives/musli_descriptive` | **160.29ns** ± 0.42ns | 159.70ns &mdash; 160.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **7.35ns** ± 0.05ns | 7.28ns &mdash; 7.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **100.19ns** ± 1.07ns | 98.68ns &mdash; 101.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_value`[^musli_value] | **359.41ns** ± 0.08ns | 359.30ns &mdash; 359.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **174.24ns** ± 1.11ns | 172.68ns &mdash; 175.79ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_wire/report/) |
| `enc/primitives/postcard` | **186.61ns** ± 0.44ns | 186.00ns &mdash; 187.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/postcard/report/) |
| `enc/primitives/serde_bincode` | **31.23ns** ± 0.34ns | 30.74ns &mdash; 31.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bincode/report/) |
| `enc/primitives/serde_bitcode` | **1.10μs** ± 3.46ns | 1.09μs &mdash; 1.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bitcode/report/) |
| `enc/primitives/serde_rmp` | **71.73ns** ± 0.04ns | 71.67ns &mdash; 71.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_rmp/report/) |


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
| `dec/primpacked/musli_descriptive` | **362.39ns** ± 1.48ns | 360.30ns &mdash; 364.48ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_packed` | **7.33ns** ± 0.05ns | 7.26ns &mdash; 7.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_packed/report/) |
| `dec/primpacked/musli_storage` | **232.55ns** ± 1.93ns | 229.81ns &mdash; 235.28ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **88.58ns** ± 0.18ns | 88.32ns &mdash; 88.84ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **279.94ns** ± 1.63ns | 277.64ns &mdash; 282.25ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/postcard` | **97.25ns** ± 0.81ns | 96.10ns &mdash; 98.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/postcard/report/) |
| `dec/primpacked/serde_bincode` | **38.57ns** ± 0.01ns | 38.56ns &mdash; 38.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_bincode/report/) |
| `dec/primpacked/serde_bitcode` | **457.26ns** ± 0.78ns | 456.16ns &mdash; 458.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_bitcode/report/) |
| `dec/primpacked/serde_rmp` | **137.95ns** ± 0.51ns | 137.22ns &mdash; 138.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primpacked/serde_rmp/report/) |

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
| `enc/primpacked/musli_descriptive` | **131.78ns** ± 0.17ns | 131.55ns &mdash; 132.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_packed` | **8.34ns** ± 0.14ns | 8.14ns &mdash; 8.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_packed/report/) |
| `enc/primpacked/musli_storage` | **99.46ns** ± 0.16ns | 99.24ns &mdash; 99.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **497.56ns** ± 4.19ns | 491.63ns &mdash; 503.48ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **116.74ns** ± 0.58ns | 115.92ns &mdash; 117.56ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/postcard` | **195.43ns** ± 1.96ns | 192.65ns &mdash; 198.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/postcard/report/) |
| `enc/primpacked/serde_bincode` | **31.90ns** ± 0.36ns | 31.39ns &mdash; 32.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_bincode/report/) |
| `enc/primpacked/serde_bitcode` | **1.40μs** ± 2.33ns | 1.39μs &mdash; 1.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_bitcode/report/) |
| `enc/primpacked/serde_rmp` | **90.78ns** ± 0.17ns | 90.54ns &mdash; 91.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primpacked/serde_rmp/report/) |


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
| `dec/medium_enum/musli_descriptive` | **753.53ns** ± 1.50ns | 751.41ns &mdash; 755.64ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_packed` | **116.97ns** ± 0.18ns | 116.72ns &mdash; 117.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_packed/report/) |
| `dec/medium_enum/musli_storage` | **490.29ns** ± 0.50ns | 489.59ns &mdash; 490.99ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **268.68ns** ± 1.01ns | 267.25ns &mdash; 270.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **548.66ns** ± 1.77ns | 546.15ns &mdash; 551.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/postcard` | **332.39ns** ± 0.15ns | 332.18ns &mdash; 332.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/postcard/report/) |
| `dec/medium_enum/serde_bincode` | **240.71ns** ± 0.77ns | 239.62ns &mdash; 241.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_bincode/report/) |
| `dec/medium_enum/serde_bitcode` | **2.57μs** ± 4.72ns | 2.56μs &mdash; 2.57μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_bitcode/report/) |
| `dec/medium_enum/serde_rmp` | **603.96ns** ± 0.23ns | 603.63ns &mdash; 604.28ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_medium_enum/serde_rmp/report/) |

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
| `enc/medium_enum/musli_descriptive` | **314.55ns** ± 0.30ns | 314.12ns &mdash; 314.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_packed` | **41.42ns** ± 0.44ns | 40.80ns &mdash; 42.04ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_packed/report/) |
| `enc/medium_enum/musli_storage` | **194.39ns** ± 1.61ns | 192.11ns &mdash; 196.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **1.17μs** ± 2.18ns | 1.16μs &mdash; 1.17μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **339.06ns** ± 0.50ns | 338.35ns &mdash; 339.77ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/postcard` | **323.46ns** ± 0.79ns | 322.35ns &mdash; 324.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/postcard/report/) |
| `enc/medium_enum/serde_bincode` | **90.38ns** ± 0.66ns | 89.45ns &mdash; 91.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_bincode/report/) |
| `enc/medium_enum/serde_bitcode` | **3.52μs** ± 6.70ns | 3.51μs &mdash; 3.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_bitcode/report/) |
| `enc/medium_enum/serde_rmp` | **211.74ns** ± 0.89ns | 210.48ns &mdash; 213.00ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_medium_enum/serde_rmp/report/) |


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
| `dec/large/musli_descriptive` | **56.88μs** ± 110.59ns | 56.73μs &mdash; 57.04μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **9.91μs** ± 21.12ns | 9.88μs &mdash; 9.94μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **37.94μs** ± 152.85ns | 37.72μs &mdash; 38.15μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_storage/report/) |
| `dec/large/musli_value`[^musli_value] | **17.55μs** ± 22.57ns | 17.51μs &mdash; 17.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **48.34μs** ± 375.22ns | 47.81μs &mdash; 48.87μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_wire/report/) |
| `dec/large/postcard` | **19.66μs** ± 48.84ns | 19.60μs &mdash; 19.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/postcard/report/) |
| `dec/large/serde_bincode` | **13.45μs** ± 20.38ns | 13.42μs &mdash; 13.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bincode/report/) |
| `dec/large/serde_bitcode` | **21.02μs** ± 51.65ns | 20.95μs &mdash; 21.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bitcode/report/) |
| `dec/large/serde_rmp` | **35.89μs** ± 620.06ns | 35.01μs &mdash; 36.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_rmp/report/) |

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
| `enc/large/musli_descriptive` | **21.37μs** ± 106.28ns | 21.22μs &mdash; 21.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **2.70μs** ± 8.85ns | 2.69μs &mdash; 2.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **15.91μs** ± 26.57ns | 15.87μs &mdash; 15.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_storage/report/) |
| `enc/large/musli_value`[^musli_value] | **92.44μs** ± 88.44ns | 92.31μs &mdash; 92.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **25.57μs** ± 46.44ns | 25.50μs &mdash; 25.63μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_wire/report/) |
| `enc/large/postcard` | **23.66μs** ± 166.95ns | 23.42μs &mdash; 23.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/postcard/report/) |
| `enc/large/serde_bincode` | **4.88μs** ± 14.98ns | 4.86μs &mdash; 4.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bincode/report/) |
| `enc/large/serde_bitcode` | **26.00μs** ± 134.80ns | 25.81μs &mdash; 26.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bitcode/report/) |
| `enc/large/serde_rmp` | **25.16μs** ± 481.96ns | 24.48μs &mdash; 25.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_rmp/report/) |


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
| `dec/allocated/musli_descriptive` | **1.07μs** ± 0.03ns | 1.07μs &mdash; 1.07μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **723.30ns** ± 1.46ns | 721.24ns &mdash; 725.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **921.92ns** ± 2.59ns | 918.26ns &mdash; 925.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_value`[^musli_value] | **610.20ns** ± 4.33ns | 604.09ns &mdash; 616.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **968.55ns** ± 3.05ns | 964.22ns &mdash; 972.88ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_wire/report/) |
| `dec/allocated/postcard` | **979.18ns** ± 0.16ns | 978.96ns &mdash; 979.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/postcard/report/) |
| `dec/allocated/serde_bincode` | **899.32ns** ± 0.77ns | 898.22ns &mdash; 900.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bincode/report/) |
| `dec/allocated/serde_bitcode` | **1.61μs** ± 25.37ns | 1.57μs &mdash; 1.64μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bitcode/report/) |
| `dec/allocated/serde_rmp` | **1.06μs** ± 4.56ns | 1.05μs &mdash; 1.06μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_rmp/report/) |

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
| `enc/allocated/musli_descriptive` | **143.08ns** ± 0.36ns | 142.57ns &mdash; 143.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **53.80ns** ± 0.02ns | 53.78ns &mdash; 53.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **123.59ns** ± 0.10ns | 123.45ns &mdash; 123.74ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_value`[^musli_value] | **712.59ns** ± 0.67ns | 711.64ns &mdash; 713.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **158.91ns** ± 0.03ns | 158.87ns &mdash; 158.94ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_wire/report/) |
| `enc/allocated/postcard` | **360.09ns** ± 0.14ns | 359.89ns &mdash; 360.28ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/postcard/report/) |
| `enc/allocated/serde_bincode` | **96.63ns** ± 0.38ns | 96.09ns &mdash; 97.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bincode/report/) |
| `enc/allocated/serde_bitcode` | **2.28μs** ± 36.10ns | 2.23μs &mdash; 2.33μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bitcode/report/) |
| `enc/allocated/serde_rmp` | **220.62ns** ± 0.88ns | 219.37ns &mdash; 221.87ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_rmp/report/) |


<table>
<tr>
<th colspan="3">
<code>full/dec/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/mesh/musli_descriptive` | **4.83μs** ± 4.37ns | 4.82μs &mdash; 4.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **30.61ns** ± 0.23ns | 30.29ns &mdash; 30.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **2.73μs** ± 11.42ns | 2.72μs &mdash; 2.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_value`[^musli_value] | **1.34μs** ± 3.67ns | 1.34μs &mdash; 1.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/musli_value/report/) |
| `dec/mesh/musli_wire` | **3.10μs** ± 8.90ns | 3.09μs &mdash; 3.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/musli_wire/report/) |
| `dec/mesh/postcard` | **307.20ns** ± 0.32ns | 306.74ns &mdash; 307.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/postcard/report/) |
| `dec/mesh/serde_bincode` | **505.72ns** ± 4.05ns | 499.99ns &mdash; 511.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/serde_bincode/report/) |
| `dec/mesh/serde_bitcode` | **1.45μs** ± 8.84ns | 1.44μs &mdash; 1.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/serde_bitcode/report/) |
| `dec/mesh/serde_rmp` | **2.05μs** ± 18.91ns | 2.02μs &mdash; 2.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/serde_rmp/report/) |

<table>
<tr>
<th colspan="3">
<code>full/enc/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/mesh/musli_descriptive` | **1.54μs** ± 16.49ns | 1.52μs &mdash; 1.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **14.30ns** ± 0.05ns | 14.24ns &mdash; 14.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **1.31μs** ± 0.74ns | 1.31μs &mdash; 1.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_value`[^musli_value] | **9.71μs** ± 20.15ns | 9.68μs &mdash; 9.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/musli_value/report/) |
| `enc/mesh/musli_wire` | **1.43μs** ± 2.79ns | 1.43μs &mdash; 1.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/musli_wire/report/) |
| `enc/mesh/postcard` | **217.50ns** ± 0.82ns | 216.34ns &mdash; 218.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/postcard/report/) |
| `enc/mesh/serde_bincode` | **309.01ns** ± 0.16ns | 308.79ns &mdash; 309.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/serde_bincode/report/) |
| `enc/mesh/serde_bitcode` | **1.92μs** ± 1.60ns | 1.91μs &mdash; 1.92μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/serde_bitcode/report/) |
| `enc/mesh/serde_rmp` | **817.85ns** ± 2.52ns | 814.30ns &mdash; 821.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/serde_rmp/report/) |



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
| `dec/primitives/musli_json` | **906.15ns** ± 1.85ns | 903.53ns &mdash; 908.76ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **1.12μs** ± 0.11ns | 1.12μs &mdash; 1.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/serde_json/report/) |

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
| `enc/primitives/musli_json` | **399.75ns** ± 2.22ns | 396.59ns &mdash; 402.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **324.41ns** ± 0.70ns | 323.42ns &mdash; 325.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/serde_json/report/) |


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
| `dec/primpacked/musli_json` | **1.07μs** ± 5.20ns | 1.06μs &mdash; 1.07μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/musli_json/report/) |
| `dec/primpacked/serde_json` | **1.19μs** ± 7.31ns | 1.18μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primpacked/serde_json/report/) |

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
| `enc/primpacked/musli_json` | **368.77ns** ± 3.86ns | 363.31ns &mdash; 374.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/musli_json/report/) |
| `enc/primpacked/serde_json` | **361.65ns** ± 0.72ns | 360.62ns &mdash; 362.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primpacked/serde_json/report/) |


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
| `dec/medium_enum/musli_json` | **2.12μs** ± 0.14ns | 2.12μs &mdash; 2.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/musli_json/report/) |
| `dec/medium_enum/serde_json` | **2.10μs** ± 36.98ns | 2.04μs &mdash; 2.15μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_medium_enum/serde_json/report/) |

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
| `enc/medium_enum/musli_json` | **790.10ns** ± 2.81ns | 786.13ns &mdash; 794.06ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/musli_json/report/) |
| `enc/medium_enum/serde_json` | **642.57ns** ± 0.67ns | 641.62ns &mdash; 643.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_medium_enum/serde_json/report/) |


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
| `dec/large/musli_json` | **177.98μs** ± 74.87ns | 177.88μs &mdash; 178.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **162.03μs** ± 938.18ns | 160.71μs &mdash; 163.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/serde_json/report/) |

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
| `enc/large/musli_json` | **55.05μs** ± 531.49ns | 54.30μs &mdash; 55.81μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **46.65μs** ± 72.19ns | 46.54μs &mdash; 46.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/serde_json/report/) |


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
| `dec/allocated/musli_json` | **2.63μs** ± 20.47ns | 2.60μs &mdash; 2.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **2.06μs** ± 6.08ns | 2.05μs &mdash; 2.07μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/serde_json/report/) |

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
| `enc/allocated/musli_json` | **698.93ns** ± 5.09ns | 691.72ns &mdash; 706.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **582.70ns** ± 0.37ns | 582.17ns &mdash; 583.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>text/dec/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/dec_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/mesh/musli_json` | **16.52μs** ± 35.01ns | 16.47μs &mdash; 16.57μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_mesh/musli_json/report/) |
| `dec/mesh/serde_json` | **13.63μs** ± 57.33ns | 13.55μs &mdash; 13.71μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_mesh/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>text/enc/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/enc_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/mesh/musli_json` | **6.92μs** ± 6.90ns | 6.91μs &mdash; 6.93μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_mesh/musli_json/report/) |
| `enc/mesh/serde_json` | **6.98μs** ± 20.18ns | 6.95μs &mdash; 7.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_mesh/serde_json/report/) |



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
| `dec/primitives/musli_descriptive` | **249.51ns** ± 1.27ns | 247.73ns &mdash; 251.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **6.14ns** ± 0.01ns | 6.13ns &mdash; 6.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **159.23ns** ± 0.23ns | 158.90ns &mdash; 159.56ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_value`[^musli_value] | **76.69ns** ± 0.68ns | 75.73ns &mdash; 77.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **169.96ns** ± 0.72ns | 168.95ns &mdash; 170.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_wire/report/) |
| `dec/primitives/serde_cbor` | **376.69ns** ± 1.42ns | 374.68ns &mdash; 378.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/serde_cbor/report/) |

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
| `enc/primitives/musli_descriptive` | **84.79ns** ± 0.19ns | 84.52ns &mdash; 85.06ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **7.41ns** ± 0.02ns | 7.38ns &mdash; 7.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **64.28ns** ± 0.03ns | 64.24ns &mdash; 64.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_value`[^musli_value] | **363.40ns** ± 2.58ns | 359.75ns &mdash; 367.05ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **140.03ns** ± 0.48ns | 139.35ns &mdash; 140.71ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_wire/report/) |
| `enc/primitives/serde_cbor` | **111.49ns** ± 1.74ns | 109.03ns &mdash; 113.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/serde_cbor/report/) |


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
| `dec/primpacked/musli_descriptive` | **262.60ns** ± 0.26ns | 262.23ns &mdash; 262.97ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_packed` | **5.55ns** ± 0.01ns | 5.54ns &mdash; 5.56ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_packed/report/) |
| `dec/primpacked/musli_storage` | **162.47ns** ± 1.35ns | 160.56ns &mdash; 164.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **78.61ns** ± 0.02ns | 78.59ns &mdash; 78.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **182.42ns** ± 0.04ns | 182.37ns &mdash; 182.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/serde_cbor` | **420.08ns** ± 3.80ns | 414.70ns &mdash; 425.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primpacked/serde_cbor/report/) |

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
| `enc/primpacked/musli_descriptive` | **86.64ns** ± 0.10ns | 86.50ns &mdash; 86.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_packed` | **6.75ns** ± 0.03ns | 6.70ns &mdash; 6.79ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_packed/report/) |
| `enc/primpacked/musli_storage` | **65.13ns** ± 0.73ns | 64.10ns &mdash; 66.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **410.85ns** ± 1.42ns | 408.84ns &mdash; 412.85ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **81.61ns** ± 0.13ns | 81.43ns &mdash; 81.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/serde_cbor` | **118.03ns** ± 0.24ns | 117.69ns &mdash; 118.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primpacked/serde_cbor/report/) |


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
| `dec/medium_enum/musli_descriptive` | **628.02ns** ± 4.52ns | 621.61ns &mdash; 634.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_packed` | **128.01ns** ± 0.23ns | 127.68ns &mdash; 128.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_packed/report/) |
| `dec/medium_enum/musli_storage` | **467.58ns** ± 0.36ns | 467.06ns &mdash; 468.09ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **264.98ns** ± 0.24ns | 264.64ns &mdash; 265.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **468.03ns** ± 0.43ns | 467.43ns &mdash; 468.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/serde_cbor` | **1.03μs** ± 11.85ns | 1.01μs &mdash; 1.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_medium_enum/serde_cbor/report/) |

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
| `enc/medium_enum/musli_descriptive` | **220.66ns** ± 0.19ns | 220.38ns &mdash; 220.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_packed` | **39.47ns** ± 0.01ns | 39.46ns &mdash; 39.48ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_packed/report/) |
| `enc/medium_enum/musli_storage` | **158.31ns** ± 0.15ns | 158.10ns &mdash; 158.51ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **1.04μs** ± 1.03ns | 1.04μs &mdash; 1.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **285.11ns** ± 0.14ns | 284.91ns &mdash; 285.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/serde_cbor` | **282.56ns** ± 1.21ns | 280.85ns &mdash; 284.27ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_medium_enum/serde_cbor/report/) |


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
| `dec/large/musli_descriptive` | **42.22μs** ± 215.57ns | 41.92μs &mdash; 42.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **3.91μs** ± 27.57ns | 3.87μs &mdash; 3.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **28.19μs** ± 93.95ns | 28.05μs &mdash; 28.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_storage/report/) |
| `dec/large/musli_value`[^musli_value] | **12.19μs** ± 155.41ns | 11.97μs &mdash; 12.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **34.66μs** ± 53.77ns | 34.59μs &mdash; 34.74μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_wire/report/) |
| `dec/large/serde_cbor` | **65.02μs** ± 317.67ns | 64.57μs &mdash; 65.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/serde_cbor/report/) |

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
| `enc/large/musli_descriptive` | **14.75μs** ± 103.78ns | 14.60μs &mdash; 14.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **1.85μs** ± 3.68ns | 1.85μs &mdash; 1.86μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **11.64μs** ± 86.37ns | 11.52μs &mdash; 11.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_storage/report/) |
| `enc/large/musli_value`[^musli_value] | **68.49μs** ± 259.24ns | 68.12μs &mdash; 68.85μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **22.86μs** ± 255.87ns | 22.50μs &mdash; 23.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_wire/report/) |
| `enc/large/serde_cbor` | **17.88μs** ± 56.16ns | 17.80μs &mdash; 17.96μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/serde_cbor/report/) |


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
| `dec/allocated/musli_descriptive` | **676.31ns** ± 1.90ns | 673.63ns &mdash; 678.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **517.12ns** ± 2.92ns | 513.00ns &mdash; 521.24ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **659.04ns** ± 1.78ns | 656.53ns &mdash; 661.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_value`[^musli_value] | **442.67ns** ± 5.57ns | 434.77ns &mdash; 450.56ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **624.55ns** ± 2.46ns | 621.06ns &mdash; 628.04ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_wire/report/) |
| `dec/allocated/serde_cbor` | **1.01μs** ± 5.21ns | 1.00μs &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/serde_cbor/report/) |

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
| `enc/allocated/musli_descriptive` | **117.11ns** ± 0.61ns | 116.25ns &mdash; 117.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **46.90ns** ± 0.24ns | 46.56ns &mdash; 47.24ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **104.07ns** ± 0.34ns | 103.59ns &mdash; 104.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_value`[^musli_value] | **598.67ns** ± 1.16ns | 597.02ns &mdash; 600.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **105.96ns** ± 0.10ns | 105.81ns &mdash; 106.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_wire/report/) |
| `enc/allocated/serde_cbor` | **192.18ns** ± 1.94ns | 189.42ns &mdash; 194.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/serde_cbor/report/) |


<table>
<tr>
<th colspan="3">
<code>fewer/dec/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/mesh/musli_descriptive` | **2.76μs** ± 20.32ns | 2.73μs &mdash; 2.79μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **25.90ns** ± 0.11ns | 25.75ns &mdash; 26.05ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **1.70μs** ± 7.15ns | 1.69μs &mdash; 1.71μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_value`[^musli_value] | **767.11ns** ± 2.11ns | 764.13ns &mdash; 770.09ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/musli_value/report/) |
| `dec/mesh/musli_wire` | **1.86μs** ± 16.82ns | 1.83μs &mdash; 1.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/musli_wire/report/) |
| `dec/mesh/serde_cbor` | **4.09μs** ± 16.89ns | 4.07μs &mdash; 4.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/serde_cbor/report/) |

<table>
<tr>
<th colspan="3">
<code>fewer/enc/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/mesh/musli_descriptive` | **942.76ns** ± 1.37ns | 940.82ns &mdash; 944.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **7.29ns** ± 0.02ns | 7.26ns &mdash; 7.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **706.86ns** ± 6.20ns | 698.08ns &mdash; 715.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_value`[^musli_value] | **5.56μs** ± 0.36ns | 5.56μs &mdash; 5.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/musli_value/report/) |
| `enc/mesh/musli_wire` | **835.83ns** ± 2.04ns | 832.95ns &mdash; 838.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/musli_wire/report/) |
| `enc/mesh/serde_cbor` | **1.98μs** ± 3.66ns | 1.97μs &mdash; 1.98μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/serde_cbor/report/) |



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
| `dec/primitives/musli_descriptive` | **327.43ns** ± 1.82ns | 324.86ns &mdash; 330.01ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **6.83ns** ± 0.00ns | 6.83ns &mdash; 6.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **206.09ns** ± 1.71ns | 203.66ns &mdash; 208.51ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_value`[^musli_value] | **96.42ns** ± 0.28ns | 96.03ns &mdash; 96.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **226.52ns** ± 1.16ns | 224.87ns &mdash; 228.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_wire/report/) |
| `dec/primitives/speedy` | **6.19ns** ± 0.02ns | 6.16ns &mdash; 6.22ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/speedy/report/) |

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
| `enc/primitives/musli_descriptive` | **136.38ns** ± 0.29ns | 135.98ns &mdash; 136.79ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **7.09ns** ± 0.02ns | 7.06ns &mdash; 7.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **89.38ns** ± 0.21ns | 89.08ns &mdash; 89.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_value`[^musli_value] | **384.94ns** ± 5.94ns | 376.54ns &mdash; 393.33ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **135.67ns** ± 0.34ns | 135.19ns &mdash; 136.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_wire/report/) |
| `enc/primitives/speedy` | **5.54ns** ± 0.00ns | 5.53ns &mdash; 5.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/speedy/report/) |


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
| `dec/primpacked/musli_descriptive` | **311.48ns** ± 0.94ns | 310.14ns &mdash; 312.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_packed` | **7.54ns** ± 0.05ns | 7.47ns &mdash; 7.62ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/musli_packed/report/) |
| `dec/primpacked/musli_storage` | **229.49ns** ± 1.78ns | 226.96ns &mdash; 232.01ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **85.47ns** ± 0.11ns | 85.32ns &mdash; 85.62ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **239.13ns** ± 3.08ns | 234.77ns &mdash; 243.50ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/musli_wire/report/) |
| `dec/primpacked/speedy` | **5.14ns** ± 0.03ns | 5.11ns &mdash; 5.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primpacked/speedy/report/) |

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
| `enc/primpacked/musli_descriptive` | **137.67ns** ± 0.23ns | 137.34ns &mdash; 137.99ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_packed` | **6.96ns** ± 0.11ns | 6.81ns &mdash; 7.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/musli_packed/report/) |
| `enc/primpacked/musli_storage` | **92.11ns** ± 0.20ns | 91.83ns &mdash; 92.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **415.61ns** ± 0.87ns | 414.38ns &mdash; 416.85ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **105.45ns** ± 0.05ns | 105.38ns &mdash; 105.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/musli_wire/report/) |
| `enc/primpacked/speedy` | **5.41ns** ± 0.01ns | 5.39ns &mdash; 5.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primpacked/speedy/report/) |


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
| `dec/medium_enum/musli_descriptive` | **715.62ns** ± 1.84ns | 713.02ns &mdash; 718.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_packed` | **156.59ns** ± 0.00ns | 156.59ns &mdash; 156.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/musli_packed/report/) |
| `dec/medium_enum/musli_storage` | **522.36ns** ± 0.87ns | 521.14ns &mdash; 523.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **307.74ns** ± 4.42ns | 301.49ns &mdash; 313.99ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **620.45ns** ± 4.70ns | 613.80ns &mdash; 627.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/musli_wire/report/) |
| `dec/medium_enum/speedy` | **182.78ns** ± 1.17ns | 181.12ns &mdash; 184.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_medium_enum/speedy/report/) |

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
| `enc/medium_enum/musli_descriptive` | **289.89ns** ± 0.36ns | 289.38ns &mdash; 290.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_packed` | **40.28ns** ± 0.39ns | 39.73ns &mdash; 40.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/musli_packed/report/) |
| `enc/medium_enum/musli_storage` | **192.52ns** ± 1.31ns | 190.67ns &mdash; 194.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **1.10μs** ± 1.62ns | 1.10μs &mdash; 1.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **292.22ns** ± 1.98ns | 289.43ns &mdash; 295.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/musli_wire/report/) |
| `enc/medium_enum/speedy` | **80.23ns** ± 0.24ns | 79.89ns &mdash; 80.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_medium_enum/speedy/report/) |


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
| `dec/large/musli_descriptive` | **58.03μs** ± 135.92ns | 57.84μs &mdash; 58.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **10.68μs** ± 35.01ns | 10.63μs &mdash; 10.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **43.19μs** ± 55.66ns | 43.11μs &mdash; 43.26μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_storage/report/) |
| `dec/large/musli_value`[^musli_value] | **18.90μs** ± 24.88ns | 18.86μs &mdash; 18.93μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **51.27μs** ± 115.98ns | 51.10μs &mdash; 51.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_wire/report/) |
| `dec/large/speedy` | **10.76μs** ± 0.49ns | 10.76μs &mdash; 10.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/speedy/report/) |

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
| `enc/large/musli_descriptive` | **23.37μs** ± 13.57ns | 23.35μs &mdash; 23.39μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **2.66μs** ± 22.69ns | 2.63μs &mdash; 2.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **16.20μs** ± 74.35ns | 16.10μs &mdash; 16.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_storage/report/) |
| `enc/large/musli_value`[^musli_value] | **92.45μs** ± 708.26ns | 91.44μs &mdash; 93.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **25.54μs** ± 424.83ns | 24.94μs &mdash; 26.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_wire/report/) |
| `enc/large/speedy` | **2.41μs** ± 8.63ns | 2.40μs &mdash; 2.42μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/speedy/report/) |


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
| `dec/allocated/musli_descriptive` | **1.12μs** ± 1.09ns | 1.12μs &mdash; 1.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **704.39ns** ± 0.81ns | 703.25ns &mdash; 705.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **1.00μs** ± 11.79ns | 988.05ns &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_value`[^musli_value] | **580.20ns** ± 2.55ns | 576.60ns &mdash; 583.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **956.11ns** ± 15.82ns | 933.67ns &mdash; 978.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_wire/report/) |
| `dec/allocated/speedy` | **851.60ns** ± 1.43ns | 849.58ns &mdash; 853.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/speedy/report/) |

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
| `enc/allocated/musli_descriptive` | **174.18ns** ± 0.27ns | 173.80ns &mdash; 174.56ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **55.32ns** ± 0.00ns | 55.31ns &mdash; 55.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **143.15ns** ± 0.56ns | 142.36ns &mdash; 143.95ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_value`[^musli_value] | **756.22ns** ± 2.74ns | 752.34ns &mdash; 760.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **157.67ns** ± 0.48ns | 156.98ns &mdash; 158.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_wire/report/) |
| `enc/allocated/speedy` | **147.62ns** ± 0.53ns | 146.87ns &mdash; 148.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/speedy/report/) |


<table>
<tr>
<th colspan="3">
<code>speedy/dec/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/mesh/musli_descriptive` | **2.45μs** ± 0.80ns | 2.45μs &mdash; 2.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **27.26ns** ± 0.10ns | 27.12ns &mdash; 27.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **1.66μs** ± 13.07ns | 1.64μs &mdash; 1.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_value`[^musli_value] | **863.64ns** ± 1.98ns | 860.83ns &mdash; 866.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/musli_value/report/) |
| `dec/mesh/musli_wire` | **1.80μs** ± 3.13ns | 1.80μs &mdash; 1.81μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/musli_wire/report/) |
| `dec/mesh/speedy` | **18.37ns** ± 0.04ns | 18.32ns &mdash; 18.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/speedy/report/) |

<table>
<tr>
<th colspan="3">
<code>speedy/enc/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/mesh/musli_descriptive` | **930.31ns** ± 0.89ns | 929.05ns &mdash; 931.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **6.58ns** ± 0.09ns | 6.45ns &mdash; 6.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **753.92ns** ± 1.17ns | 752.27ns &mdash; 755.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_value`[^musli_value] | **5.67μs** ± 9.25ns | 5.66μs &mdash; 5.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/musli_value/report/) |
| `enc/mesh/musli_wire` | **1.16μs** ± 2.49ns | 1.15μs &mdash; 1.16μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/musli_wire/report/) |
| `enc/mesh/speedy` | **11.96ns** ± 0.04ns | 11.90ns &mdash; 12.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/speedy/report/) |



### ε-serde

> **Custom environment:**
> - `MUSLI_VEC_RANGE=10000..20000` - ε-serde benefits from larger inputs, this ensures that the size of the supported suite (primarily `mesh`) reflects that by making the inputs bigger.


This is a test suite for ε-serde features

Since ε-serde works best for larger inputs,
we increase the size of the input being deserialized.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/report/)
* [Sizes](#ε-serde-sizes)

<table>
<tr>
<th colspan="3">
<code>epserde/dec/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primitives/epserde` | **543.18ns** ± 0.53ns | 542.43ns &mdash; 543.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/epserde/report/) |
| `dec/primitives/musli_descriptive` | **353.16ns** ± 1.86ns | 350.53ns &mdash; 355.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **7.33ns** ± 0.02ns | 7.31ns &mdash; 7.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **223.07ns** ± 1.19ns | 221.39ns &mdash; 224.76ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_value`[^musli_value] | **86.27ns** ± 1.08ns | 84.74ns &mdash; 87.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **238.98ns** ± 0.79ns | 237.86ns &mdash; 240.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>epserde/enc/primitives</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primitives/epserde` | **499.16ns** ± 3.73ns | 493.89ns &mdash; 504.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/epserde/report/) |
| `enc/primitives/musli_descriptive` | **162.92ns** ± 1.75ns | 160.45ns &mdash; 165.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **8.06ns** ± 0.03ns | 8.02ns &mdash; 8.09ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **98.95ns** ± 1.34ns | 97.05ns &mdash; 100.86ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_value`[^musli_value] | **386.39ns** ± 1.68ns | 384.01ns &mdash; 388.77ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **172.10ns** ± 0.74ns | 171.05ns &mdash; 173.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>epserde/dec/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/primpacked/epserde` | **616.82ns** ± 0.28ns | 616.42ns &mdash; 617.22ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primpacked/epserde/report/) |
| `dec/primpacked/musli_descriptive` | **352.93ns** ± 0.55ns | 352.15ns &mdash; 353.71ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_packed` | **1.26ns** ± 0.00ns | 1.26ns &mdash; 1.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primpacked/musli_packed/report/) |
| `dec/primpacked/musli_storage` | **221.73ns** ± 0.17ns | 221.48ns &mdash; 221.97ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_value`[^musli_value] | **83.46ns** ± 0.56ns | 82.66ns &mdash; 84.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primpacked/musli_value/report/) |
| `dec/primpacked/musli_wire` | **247.54ns** ± 0.18ns | 247.28ns &mdash; 247.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primpacked/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>epserde/enc/primpacked</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primpacked/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/primpacked/epserde` | **574.76ns** ± 0.02ns | 574.73ns &mdash; 574.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primpacked/epserde/report/) |
| `enc/primpacked/musli_descriptive` | **132.66ns** ± 0.11ns | 132.51ns &mdash; 132.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_packed` | **2.10ns** ± 0.01ns | 2.09ns &mdash; 2.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primpacked/musli_packed/report/) |
| `enc/primpacked/musli_storage` | **99.09ns** ± 0.70ns | 98.10ns &mdash; 100.09ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_value`[^musli_value] | **496.91ns** ± 2.52ns | 493.36ns &mdash; 500.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primpacked/musli_value/report/) |
| `enc/primpacked/musli_wire` | **113.79ns** ± 0.15ns | 113.57ns &mdash; 114.00ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primpacked/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>epserde/dec/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/medium_enum/musli_descriptive` | **74.32μs** ± 118.43ns | 74.15μs &mdash; 74.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_packed` | **312.37ns** ± 2.58ns | 308.72ns &mdash; 316.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_medium_enum/musli_packed/report/) |
| `dec/medium_enum/musli_storage` | **11.74μs** ± 12.58ns | 11.73μs &mdash; 11.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_value`[^musli_value] | **23.42μs** ± 5.92ns | 23.41μs &mdash; 23.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_medium_enum/musli_value/report/) |
| `dec/medium_enum/musli_wire` | **62.11μs** ± 1.12μs | 60.53μs &mdash; 63.69μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_medium_enum/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>epserde/enc/medium_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_medium_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/medium_enum/musli_descriptive` | **44.32μs** ± 105.15ns | 44.17μs &mdash; 44.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_packed` | **118.94ns** ± 0.23ns | 118.62ns &mdash; 119.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_medium_enum/musli_packed/report/) |
| `enc/medium_enum/musli_storage` | **5.70μs** ± 3.67ns | 5.69μs &mdash; 5.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_value`[^musli_value] | **94.83μs** ± 85.30ns | 94.71μs &mdash; 94.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_medium_enum/musli_value/report/) |
| `enc/medium_enum/musli_wire` | **44.61μs** ± 65.25ns | 44.52μs &mdash; 44.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_medium_enum/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>epserde/dec/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/large/musli_descriptive` | **1.99ms** ± 1.30μs | 1.99ms &mdash; 1.99ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **61.62μs** ± 344.39ns | 61.14μs &mdash; 62.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **728.80μs** ± 1.86μs | 726.17μs &mdash; 731.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_large/musli_storage/report/) |
| `dec/large/musli_value`[^musli_value] | **558.32μs** ± 2.26μs | 555.12μs &mdash; 561.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **1.77ms** ± 3.19μs | 1.76ms &mdash; 1.77ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_large/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>epserde/enc/large</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_large/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/large/musli_descriptive` | **1.04ms** ± 2.36μs | 1.04ms &mdash; 1.04ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **38.70μs** ± 11.54ns | 38.68μs &mdash; 38.71μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **330.59μs** ± 290.87ns | 330.18μs &mdash; 331.00μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_large/musli_storage/report/) |
| `enc/large/musli_value`[^musli_value] | **2.45ms** ± 19.98μs | 2.43ms &mdash; 2.48ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **1.18ms** ± 5.01μs | 1.17ms &mdash; 1.18ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_large/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>epserde/dec/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/allocated/musli_descriptive` | **1.34μs** ± 2.24ns | 1.34μs &mdash; 1.34μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **934.52ns** ± 0.72ns | 933.49ns &mdash; 935.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **1.22μs** ± 0.80ns | 1.22μs &mdash; 1.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_value`[^musli_value] | **693.67ns** ± 0.85ns | 692.47ns &mdash; 694.86ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **1.27μs** ± 2.26ns | 1.27μs &mdash; 1.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_allocated/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>epserde/enc/allocated</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_allocated/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/allocated/musli_descriptive` | **188.67ns** ± 0.08ns | 188.56ns &mdash; 188.79ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **62.33ns** ± 0.04ns | 62.27ns &mdash; 62.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **154.58ns** ± 0.24ns | 154.23ns &mdash; 154.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_value`[^musli_value] | **812.61ns** ± 5.19ns | 805.29ns &mdash; 819.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **212.12ns** ± 0.12ns | 211.95ns &mdash; 212.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_allocated/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>epserde/dec/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/mesh/epserde` | **665.70ns** ± 0.66ns | 664.76ns &mdash; 666.64ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/epserde/report/) |
| `dec/mesh/musli_descriptive` | **3.84ms** ± 460.25ns | 3.84ms &mdash; 3.84ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **32.82μs** ± 192.34ns | 32.55μs &mdash; 33.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **2.44ms** ± 5.67μs | 2.43ms &mdash; 2.44ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_value`[^musli_value] | **1.60ms** ± 3.70μs | 1.60ms &mdash; 1.61ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/musli_value/report/) |
| `dec/mesh/musli_wire` | **2.71ms** ± 3.72μs | 2.70ms &mdash; 2.71ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>epserde/enc/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/mesh/epserde` | **22.60μs** ± 18.68ns | 22.57μs &mdash; 22.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/epserde/report/) |
| `enc/mesh/musli_descriptive` | **1.29ms** ± 3.11μs | 1.29ms &mdash; 1.30ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **22.44μs** ± 225.06ns | 22.12μs &mdash; 22.76μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **1.06ms** ± 1.88μs | 1.06ms &mdash; 1.06ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_value`[^musli_value] | **12.36ms** ± 14.60μs | 12.34ms &mdash; 12.38ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/musli_value/report/) |
| `enc/mesh/musli_wire` | **1.21ms** ± 8.16μs | 1.20ms &mdash; 1.23ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/musli_wire/report/) |



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
| `dec/primpacked/musli_zerocopy` | **1.30ns** ± 0.00ns | 1.29ns &mdash; 1.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/musli_zerocopy/report/) |
| `dec/primpacked/zerocopy` | **1.68ns** ± 0.00ns | 1.68ns &mdash; 1.68ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_primpacked/zerocopy/report/) |

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
| `enc/primpacked/musli_zerocopy` | **4.96ns** ± 0.00ns | 4.95ns &mdash; 4.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/musli_zerocopy/report/) |
| `enc/primpacked/zerocopy` | **2.17ns** ± 0.04ns | 2.12ns &mdash; 2.22ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_primpacked/zerocopy/report/) |



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
| `dec/primitives/derive_bitcode` | **75.72ns** ± 0.54ns | 74.95ns &mdash; 76.49ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/derive_bitcode/report/) |
| `dec/primitives/musli_descriptive` | **343.29ns** ± 3.55ns | 338.25ns &mdash; 348.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **6.91ns** ± 0.01ns | 6.90ns &mdash; 6.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **219.61ns** ± 0.11ns | 219.45ns &mdash; 219.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_wire` | **234.63ns** ± 0.84ns | 233.44ns &mdash; 235.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/derive_bitcode` | **408.97ns** ± 0.85ns | 407.76ns &mdash; 410.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/derive_bitcode/report/) |
| `enc/primitives/musli_descriptive` | **159.75ns** ± 0.90ns | 158.48ns &mdash; 161.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **7.26ns** ± 0.05ns | 7.20ns &mdash; 7.33ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **101.56ns** ± 1.34ns | 99.67ns &mdash; 103.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_wire` | **188.15ns** ± 0.35ns | 187.65ns &mdash; 188.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_wire/report/) |


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
| `dec/primpacked/derive_bitcode` | **72.48ns** ± 0.65ns | 71.56ns &mdash; 73.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/derive_bitcode/report/) |
| `dec/primpacked/musli_descriptive` | **349.45ns** ± 1.36ns | 347.53ns &mdash; 351.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_packed` | **7.23ns** ± 0.01ns | 7.22ns &mdash; 7.25ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_packed/report/) |
| `dec/primpacked/musli_storage` | **225.40ns** ± 0.80ns | 224.27ns &mdash; 226.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_wire` | **240.95ns** ± 0.48ns | 240.27ns &mdash; 241.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primpacked/musli_wire/report/) |

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
| `enc/primpacked/derive_bitcode` | **422.81ns** ± 1.37ns | 420.87ns &mdash; 424.74ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/derive_bitcode/report/) |
| `enc/primpacked/musli_descriptive` | **148.42ns** ± 1.52ns | 146.26ns &mdash; 150.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_packed` | **8.40ns** ± 0.05ns | 8.34ns &mdash; 8.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_packed/report/) |
| `enc/primpacked/musli_storage` | **101.05ns** ± 0.19ns | 100.78ns &mdash; 101.33ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_wire` | **113.00ns** ± 0.26ns | 112.64ns &mdash; 113.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primpacked/musli_wire/report/) |


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
| `dec/medium_enum/derive_bitcode` | **903.61ns** ± 6.45ns | 894.44ns &mdash; 912.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/derive_bitcode/report/) |
| `dec/medium_enum/musli_descriptive` | **767.99ns** ± 5.76ns | 759.84ns &mdash; 776.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_packed` | **140.16ns** ± 0.89ns | 138.89ns &mdash; 141.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_packed/report/) |
| `dec/medium_enum/musli_storage` | **495.97ns** ± 1.32ns | 494.10ns &mdash; 497.84ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_wire` | **578.04ns** ± 0.89ns | 576.78ns &mdash; 579.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_medium_enum/musli_wire/report/) |

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
| `enc/medium_enum/derive_bitcode` | **4.22μs** ± 16.77ns | 4.19μs &mdash; 4.24μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/derive_bitcode/report/) |
| `enc/medium_enum/musli_descriptive` | **298.69ns** ± 2.20ns | 295.59ns &mdash; 301.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_packed` | **38.33ns** ± 0.04ns | 38.27ns &mdash; 38.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_packed/report/) |
| `enc/medium_enum/musli_storage` | **192.42ns** ± 0.22ns | 192.11ns &mdash; 192.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_wire` | **344.84ns** ± 2.00ns | 342.01ns &mdash; 347.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_medium_enum/musli_wire/report/) |


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
| `dec/large/derive_bitcode` | **8.77μs** ± 29.73ns | 8.73μs &mdash; 8.81μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/derive_bitcode/report/) |
| `dec/large/musli_descriptive` | **58.11μs** ± 183.17ns | 57.85μs &mdash; 58.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **11.27μs** ± 15.12ns | 11.25μs &mdash; 11.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **41.37μs** ± 95.22ns | 41.24μs &mdash; 41.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_storage/report/) |
| `dec/large/musli_wire` | **48.00μs** ± 136.11ns | 47.81μs &mdash; 48.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_wire/report/) |

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
| `enc/large/derive_bitcode` | **17.76μs** ± 78.03ns | 17.65μs &mdash; 17.87μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/derive_bitcode/report/) |
| `enc/large/musli_descriptive` | **21.48μs** ± 55.18ns | 21.41μs &mdash; 21.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **2.61μs** ± 4.57ns | 2.60μs &mdash; 2.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **15.07μs** ± 73.92ns | 14.97μs &mdash; 15.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_storage/report/) |
| `enc/large/musli_wire` | **25.74μs** ± 265.85ns | 25.36μs &mdash; 26.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_wire/report/) |


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
| `dec/allocated/derive_bitcode` | **1.03μs** ± 1.37ns | 1.02μs &mdash; 1.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/derive_bitcode/report/) |
| `dec/allocated/musli_descriptive` | **1.11μs** ± 1.93ns | 1.11μs &mdash; 1.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **782.97ns** ± 2.95ns | 778.78ns &mdash; 787.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **1.01μs** ± 4.39ns | 1.00μs &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_wire` | **1.01μs** ± 13.63ns | 989.46ns &mdash; 1.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_wire/report/) |

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
| `enc/allocated/derive_bitcode` | **1.65μs** ± 2.19ns | 1.65μs &mdash; 1.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/derive_bitcode/report/) |
| `enc/allocated/musli_descriptive` | **141.56ns** ± 0.05ns | 141.48ns &mdash; 141.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **54.20ns** ± 0.04ns | 54.15ns &mdash; 54.25ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **110.80ns** ± 0.14ns | 110.59ns &mdash; 111.00ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_wire` | **135.04ns** ± 0.02ns | 135.02ns &mdash; 135.07ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bitcode-derive/dec/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/mesh/derive_bitcode` | **140.08ns** ± 0.16ns | 139.85ns &mdash; 140.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/derive_bitcode/report/) |
| `dec/mesh/musli_descriptive` | **3.96μs** ± 4.65ns | 3.96μs &mdash; 3.97μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **30.08ns** ± 0.01ns | 30.07ns &mdash; 30.10ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **2.47μs** ± 1.54ns | 2.47μs &mdash; 2.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_wire` | **2.62μs** ± 0.33ns | 2.62μs &mdash; 2.63μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bitcode-derive/enc/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/mesh/derive_bitcode` | **683.04ns** ± 0.22ns | 682.74ns &mdash; 683.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/derive_bitcode/report/) |
| `enc/mesh/musli_descriptive` | **1.38μs** ± 4.43ns | 1.37μs &mdash; 1.38μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **9.78ns** ± 0.07ns | 9.67ns &mdash; 9.88ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **1.13μs** ± 1.49ns | 1.13μs &mdash; 1.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_wire` | **1.27μs** ± 8.50ns | 1.25μs &mdash; 1.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/musli_wire/report/) |



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
| `dec/primitives/bson`[^bson] | **731.96ns** ± 0.17ns | 731.72ns &mdash; 732.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/bson/report/) |
| `dec/primitives/musli_descriptive` | **254.71ns** ± 0.09ns | 254.58ns &mdash; 254.84ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **5.90ns** ± 0.03ns | 5.86ns &mdash; 5.94ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **156.97ns** ± 2.58ns | 153.30ns &mdash; 160.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_wire` | **166.53ns** ± 0.52ns | 165.79ns &mdash; 167.27ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/bson`[^bson] | **402.37ns** ± 1.21ns | 400.67ns &mdash; 404.08ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/bson/report/) |
| `enc/primitives/musli_descriptive` | **84.00ns** ± 0.22ns | 83.68ns &mdash; 84.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **7.02ns** ± 0.00ns | 7.02ns &mdash; 7.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **67.10ns** ± 0.38ns | 66.56ns &mdash; 67.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_wire` | **127.82ns** ± 0.39ns | 127.26ns &mdash; 128.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_wire/report/) |


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
| `dec/primpacked/bson`[^bson] | **1.03μs** ± 11.28ns | 1.02μs &mdash; 1.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/bson/report/) |
| `dec/primpacked/musli_descriptive` | **261.33ns** ± 0.16ns | 261.10ns &mdash; 261.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_descriptive/report/) |
| `dec/primpacked/musli_packed` | **5.57ns** ± 0.01ns | 5.56ns &mdash; 5.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_packed/report/) |
| `dec/primpacked/musli_storage` | **147.20ns** ± 0.34ns | 146.72ns &mdash; 147.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_storage/report/) |
| `dec/primpacked/musli_wire` | **170.33ns** ± 0.81ns | 169.18ns &mdash; 171.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primpacked/musli_wire/report/) |

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
| `enc/primpacked/bson`[^bson] | **689.22ns** ± 0.19ns | 688.94ns &mdash; 689.49ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/bson/report/) |
| `enc/primpacked/musli_descriptive` | **84.12ns** ± 0.42ns | 83.52ns &mdash; 84.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_descriptive/report/) |
| `enc/primpacked/musli_packed` | **7.27ns** ± 0.02ns | 7.25ns &mdash; 7.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_packed/report/) |
| `enc/primpacked/musli_storage` | **60.87ns** ± 0.03ns | 60.82ns &mdash; 60.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_storage/report/) |
| `enc/primpacked/musli_wire` | **77.03ns** ± 0.28ns | 76.64ns &mdash; 77.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primpacked/musli_wire/report/) |


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
| `dec/medium_enum/bson`[^bson] | **2.13μs** ± 0.81ns | 2.12μs &mdash; 2.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/bson/report/) |
| `dec/medium_enum/musli_descriptive` | **590.63ns** ± 0.63ns | 589.73ns &mdash; 591.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_descriptive/report/) |
| `dec/medium_enum/musli_packed` | **113.93ns** ± 0.15ns | 113.71ns &mdash; 114.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_packed/report/) |
| `dec/medium_enum/musli_storage` | **376.26ns** ± 0.97ns | 374.89ns &mdash; 377.62ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_storage/report/) |
| `dec/medium_enum/musli_wire` | **446.64ns** ± 2.16ns | 443.59ns &mdash; 449.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_medium_enum/musli_wire/report/) |

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
| `enc/medium_enum/bson`[^bson] | **1.53μs** ± 13.40ns | 1.51μs &mdash; 1.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/bson/report/) |
| `enc/medium_enum/musli_descriptive` | **185.08ns** ± 0.81ns | 183.94ns &mdash; 186.22ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_descriptive/report/) |
| `enc/medium_enum/musli_packed` | **27.93ns** ± 0.02ns | 27.90ns &mdash; 27.95ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_packed/report/) |
| `enc/medium_enum/musli_storage` | **130.73ns** ± 0.01ns | 130.72ns &mdash; 130.74ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_storage/report/) |
| `enc/medium_enum/musli_wire` | **250.61ns** ± 0.93ns | 249.30ns &mdash; 251.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_medium_enum/musli_wire/report/) |


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
| `dec/large/bson`[^bson] | **577.89μs** ± 2.57μs | 574.26μs &mdash; 581.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/bson/report/) |
| `dec/large/musli_descriptive` | **151.41μs** ± 0.67ns | 151.41μs &mdash; 151.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **31.91μs** ± 66.12ns | 31.82μs &mdash; 32.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **104.25μs** ± 1.26μs | 102.47μs &mdash; 106.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_storage/report/) |
| `dec/large/musli_wire` | **136.14μs** ± 424.37ns | 135.54μs &mdash; 136.74μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_wire/report/) |

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
| `enc/large/bson`[^bson] | **361.70μs** ± 2.71μs | 357.88μs &mdash; 365.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/bson/report/) |
| `enc/large/musli_descriptive` | **50.17μs** ± 312.66ns | 49.72μs &mdash; 50.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **9.05μs** ± 7.43ns | 9.04μs &mdash; 9.06μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **34.48μs** ± 58.58ns | 34.39μs &mdash; 34.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_storage/report/) |
| `enc/large/musli_wire` | **63.37μs** ± 84.67ns | 63.25μs &mdash; 63.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_wire/report/) |


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
| `dec/allocated/bson`[^bson] | **1.89μs** ± 8.84ns | 1.88μs &mdash; 1.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/bson/report/) |
| `dec/allocated/musli_descriptive` | **1.03μs** ± 4.19ns | 1.03μs &mdash; 1.04μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **771.93ns** ± 2.04ns | 769.04ns &mdash; 774.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **954.95ns** ± 0.48ns | 954.27ns &mdash; 955.62ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_wire` | **952.41ns** ± 1.43ns | 950.40ns &mdash; 954.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_wire/report/) |

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
| `enc/allocated/bson`[^bson] | **703.37ns** ± 7.18ns | 693.21ns &mdash; 713.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/bson/report/) |
| `enc/allocated/musli_descriptive` | **106.61ns** ± 0.69ns | 105.63ns &mdash; 107.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **44.74ns** ± 0.37ns | 44.21ns &mdash; 45.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **92.73ns** ± 0.00ns | 92.72ns &mdash; 92.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_wire` | **99.08ns** ± 0.18ns | 98.81ns &mdash; 99.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bson/dec/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/mesh/bson`[^bson] | **8.88μs** ± 119.49ns | 8.71μs &mdash; 9.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/bson/report/) |
| `dec/mesh/musli_descriptive` | **2.63μs** ± 6.66ns | 2.62μs &mdash; 2.64μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **28.75ns** ± 0.31ns | 28.31ns &mdash; 29.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **1.68μs** ± 3.50ns | 1.67μs &mdash; 1.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_wire` | **1.80μs** ± 7.57ns | 1.79μs &mdash; 1.81μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bson/enc/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/mesh/bson`[^bson] | **3.13μs** ± 7.56ns | 3.12μs &mdash; 3.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/bson/report/) |
| `enc/mesh/musli_descriptive` | **848.39ns** ± 0.30ns | 847.97ns &mdash; 848.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **8.13ns** ± 0.01ns | 8.11ns &mdash; 8.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **750.34ns** ± 0.99ns | 748.95ns &mdash; 751.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_wire` | **780.08ns** ± 6.20ns | 771.30ns &mdash; 788.86ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/musli_wire/report/) |



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
| `dec/primitives/miniserde` | **465.99ns** ± 1.97ns | 463.21ns &mdash; 468.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/miniserde/report/) |
| `dec/primitives/musli_json` | **626.55ns** ± 3.66ns | 621.38ns &mdash; 631.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **535.83ns** ± 5.73ns | 527.73ns &mdash; 543.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/serde_json/report/) |

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
| `enc/primitives/miniserde` | **636.36ns** ± 2.03ns | 633.49ns &mdash; 639.22ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/miniserde/report/) |
| `enc/primitives/musli_json` | **281.16ns** ± 0.05ns | 281.09ns &mdash; 281.22ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **273.81ns** ± 3.70ns | 268.59ns &mdash; 279.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/serde_json/report/) |


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
| `dec/primpacked/miniserde` | **631.57ns** ± 0.59ns | 630.73ns &mdash; 632.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/miniserde/report/) |
| `dec/primpacked/musli_json` | **853.93ns** ± 6.49ns | 844.76ns &mdash; 863.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/musli_json/report/) |
| `dec/primpacked/serde_json` | **684.63ns** ± 3.28ns | 679.99ns &mdash; 689.28ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primpacked/serde_json/report/) |

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
| `enc/primpacked/miniserde` | **912.67ns** ± 10.87ns | 897.33ns &mdash; 928.01ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/miniserde/report/) |
| `enc/primpacked/musli_json` | **289.69ns** ± 0.12ns | 289.52ns &mdash; 289.86ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/musli_json/report/) |
| `enc/primpacked/serde_json` | **309.38ns** ± 1.62ns | 307.10ns &mdash; 311.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primpacked/serde_json/report/) |


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
| `dec/medium_enum/miniserde` | **19.88ns** ± 0.01ns | 19.87ns &mdash; 19.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/miniserde/report/) |
| `dec/medium_enum/musli_json` | **18.51ns** ± 0.20ns | 18.23ns &mdash; 18.79ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/musli_json/report/) |
| `dec/medium_enum/serde_json` | **24.75ns** ± 0.33ns | 24.29ns &mdash; 25.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_medium_enum/serde_json/report/) |

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
| `enc/medium_enum/miniserde` | **27.99ns** ± 0.01ns | 27.98ns &mdash; 27.99ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/miniserde/report/) |
| `enc/medium_enum/musli_json` | **7.08ns** ± 0.02ns | 7.05ns &mdash; 7.12ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/musli_json/report/) |
| `enc/medium_enum/serde_json` | **9.07ns** ± 0.05ns | 9.00ns &mdash; 9.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_medium_enum/serde_json/report/) |


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
| `dec/large/miniserde` | **29.54μs** ± 64.69ns | 29.45μs &mdash; 29.63μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/miniserde/report/) |
| `dec/large/musli_json` | **45.25μs** ± 144.01ns | 45.04μs &mdash; 45.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **37.49μs** ± 49.67ns | 37.42μs &mdash; 37.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/serde_json/report/) |

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
| `enc/large/miniserde` | **27.94μs** ± 28.02ns | 27.90μs &mdash; 27.98μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/miniserde/report/) |
| `enc/large/musli_json` | **19.35μs** ± 254.41ns | 18.99μs &mdash; 19.71μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **17.57μs** ± 27.00ns | 17.53μs &mdash; 17.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/serde_json/report/) |


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
| `dec/allocated/miniserde` | **200.70ns** ± 0.71ns | 199.70ns &mdash; 201.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/miniserde/report/) |
| `dec/allocated/musli_json` | **191.20ns** ± 1.77ns | 188.70ns &mdash; 193.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **150.12ns** ± 0.10ns | 149.98ns &mdash; 150.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/serde_json/report/) |

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
| `enc/allocated/miniserde` | **228.28ns** ± 1.27ns | 226.49ns &mdash; 230.08ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/miniserde/report/) |
| `enc/allocated/musli_json` | **41.84ns** ± 0.07ns | 41.74ns &mdash; 41.94ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **46.37ns** ± 0.01ns | 46.36ns &mdash; 46.38ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>miniserde/dec/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_mesh_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/mesh/miniserde` | **6.96μs** ± 40.24ns | 6.90μs &mdash; 7.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_mesh/miniserde/report/) |
| `dec/mesh/musli_json` | **8.52μs** ± 31.75ns | 8.48μs &mdash; 8.57μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_mesh/musli_json/report/) |
| `dec/mesh/serde_json` | **7.18μs** ± 21.23ns | 7.15μs &mdash; 7.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_mesh/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>miniserde/enc/mesh</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_mesh/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_mesh_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/mesh/miniserde` | **7.69μs** ± 17.70ns | 7.67μs &mdash; 7.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_mesh/miniserde/report/) |
| `enc/mesh/musli_json` | **3.36μs** ± 18.80ns | 3.33μs &mdash; 3.38μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_mesh/musli_json/report/) |
| `enc/mesh/serde_json` | **3.56μs** ± 31.14ns | 3.52μs &mdash; 3.60μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_mesh/serde_json/report/) |



## Size comparisons

This is not yet an area which has received much focus, but because people are bound to ask the following section performs a raw size comparison between different formats.
Each test suite serializes a collection of values, which have all been randomly populated.
- A small object containing one of each primitive type and a string and a byte array. (`primitives`)
- Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries. (`primpacked`)
- A moderately sized enum with every kind of supported variant. (`medium_enum`)
- A really big and complex struct. (`large`)
- A sparse struct which contains fairly plain allocated data like strings and vectors. (`allocated`)
- A mesh containing triangles. (`mesh`)

> **Note** so far these are all synthetic examples. Real world data is
> rarely *this* random. But hopefully it should give an idea of the extreme
> ranges.

#### Full features sizes

These frameworks provide a fair comparison against Müsli on various areas since
they support the same set of features in what types of data they can represent.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `musli_descriptive` | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 25295, max: 61463, stddev: 11847.805317863726">49647.50 ± 11847.81</a> | <a title="samples: 100, min: 447, max: 887, stddev: 103.34544934345199">645.91 ± 103.35</a> | <a title="samples: 4000, min: 4, max: 190, stddev: 65.04924003975692">54.03 ± 65.05</a> | <a title="samples: 10, min: 1094, max: 1748, stddev: 237.80992409905858">1442.80 ± 237.81</a> |
| `musli_packed` | <a title="samples: 500, min: 95, max: 95, stddev: 0">95.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 22958, max: 63679, stddev: 12161.381693294557">48955.90 ± 12161.38</a> | <a title="samples: 100, min: 505, max: 963, stddev: 105.73878191089588">708.50 ± 105.74</a> | <a title="samples: 4000, min: 16, max: 191, stddev: 54.14185957048389">59.62 ± 54.14</a> | <a title="samples: 10, min: 488, max: 776, stddev: 104.72363630050286">641.60 ± 104.72</a> |
| `musli_storage` | <a title="samples: 500, min: 122, max: 131, stddev: 1.3556681009745792">127.86 ± 1.36</a> | <a title="samples: 500, min: 127, max: 134, stddev: 1.3827783625729677">130.88 ± 1.38</a> | <a title="samples: 10, min: 21145, max: 49729, stddev: 9297.125779508418">40343.20 ± 9297.13</a> | <a title="samples: 100, min: 419, max: 854, stddev: 101.93807139631396">616.64 ± 101.94</a> | <a title="samples: 4000, min: 2, max: 151, stddev: 53.034005312040286">43.58 ± 53.03</a> | <a title="samples: 10, min: 813, max: 1299, stddev: 176.72113625709858">1072.20 ± 176.72</a> |
| `musli_wire` | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 23533, max: 57105, stddev: 10938.562155969128">46124.40 ± 10938.56</a> | <a title="samples: 100, min: 432, max: 873, stddev: 103.44413903165322">632.99 ± 103.44</a> | <a title="samples: 4000, min: 3, max: 179, stddev: 59.57751017791893">49.59 ± 59.58</a> | <a title="samples: 10, min: 933, max: 1491, stddev: 202.90204533222428">1230.60 ± 202.90</a> |
| `postcard` | <a title="samples: 500, min: 105, max: 114, stddev: 1.4079360780944647">110.85 ± 1.41</a> | <a title="samples: 500, min: 107, max: 114, stddev: 1.3359101766211645">110.81 ± 1.34</a> | <a title="samples: 10, min: 18832, max: 41860, stddev: 7489.294573589692">34429.30 ± 7489.29</a> | <a title="samples: 100, min: 406, max: 841, stddev: 101.93807139631396">603.64 ± 101.94</a> | <a title="samples: 4000, min: 1, max: 146, stddev: 48.185339854954464">39.82 ± 48.19</a> | <a title="samples: 10, min: 481, max: 769, stddev: 104.72363630050286">634.60 ± 104.72</a> |
| `serde_bincode` | <a title="samples: 500, min: 93, max: 95, stddev: 0.20591260281973842">94.96 ± 0.21</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 18076, max: 39362, stddev: 6572.198284440299">32140.10 ± 6572.20</a> | <a title="samples: 100, min: 504, max: 962, stddev: 105.73878191089588">707.50 ± 105.74</a> | <a title="samples: 4000, min: 4, max: 163, stddev: 47.42856512429996">42.61 ± 47.43</a> | <a title="samples: 10, min: 488, max: 776, stddev: 104.72363630050286">641.60 ± 104.72</a> |
| `serde_bitcode` | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 105, max: 105, stddev: 0">105.00 ± 0.00</a> | <a title="samples: 10, min: 16856, max: 35939, stddev: 6106.095791092701">29713.30 ± 6106.10</a> | <a title="samples: 100, min: 401, max: 838, stddev: 101.51465854742356">597.71 ± 101.51</a> | <a title="samples: 4000, min: 1, max: 147, stddev: 47.011009750376175">38.98 ± 47.01</a> | <a title="samples: 10, min: 481, max: 769, stddev: 104.72363630050286">634.60 ± 104.72</a> |
| `serde_rmp` | <a title="samples: 500, min: 111, max: 115, stddev: 0.7291090453423233">113.82 ± 0.73</a> | <a title="samples: 500, min: 116, max: 123, stddev: 1.4824304368165206">119.88 ± 1.48</a> | <a title="samples: 10, min: 20714, max: 48016, stddev: 8571.98435661195">38904.70 ± 8571.98</a> | <a title="samples: 100, min: 412, max: 851, stddev: 102.82278492629929">609.57 ± 102.82</a> | <a title="samples: 4000, min: 6, max: 173, stddev: 50.88175615827298">51.21 ± 50.88</a> | <a title="samples: 10, min: 652, max: 1044, stddev: 142.58415059185225">860.60 ± 142.58</a> |

#### Text-based formats sizes

These are text-based formats, which support the full feature set of this test suite.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `musli_json`[^incomplete] | <a title="samples: 500, min: 428, max: 442, stddev: 2.370359466410104">435.41 ± 2.37</a> | <a title="samples: 500, min: 443, max: 460, stddev: 2.9921657708088594">452.29 ± 2.99</a> | <a title="samples: 10, min: 53818, max: 135258, stddev: 26708.92076460597">105772.30 ± 26708.92</a> | <a title="samples: 100, min: 695, max: 1223, stddev: 116.12703216736405">935.82 ± 116.13</a> | <a title="samples: 4000, min: 12, max: 508, stddev: 154.99859129823562">109.97 ± 155.00</a> | <a title="samples: 10, min: 2114, max: 3369, stddev: 457.93677511202355">2780.10 ± 457.94</a> |
| `serde_json`[^incomplete] | <a title="samples: 500, min: 428, max: 442, stddev: 2.370359466410104">435.41 ± 2.37</a> | <a title="samples: 500, min: 443, max: 460, stddev: 2.9921657708088594">452.29 ± 2.99</a> | <a title="samples: 10, min: 53655, max: 135057, stddev: 26723.303290573942">105487.20 ± 26723.30</a> | <a title="samples: 100, min: 693, max: 1221, stddev: 116.12703216736405">933.82 ± 116.13</a> | <a title="samples: 4000, min: 7, max: 508, stddev: 155.48331177955228">107.34 ± 155.48</a> | <a title="samples: 10, min: 2114, max: 3369, stddev: 457.93677511202355">2780.10 ± 457.94</a> |

#### Fewer features sizes

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `map` - Maps like `MashMap<K, V>` are not supported.

This is a suite where support for 128-bit integers and maps are disabled.
Usually because the underlying framework lacks support for them.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `musli_descriptive` | <a title="samples: 500, min: 112, max: 120, stddev: 1.4613363746926964">116.36 ± 1.46</a> | <a title="samples: 500, min: 118, max: 126, stddev: 1.457772273024832">122.33 ± 1.46</a> | <a title="samples: 10, min: 14103, max: 58953, stddev: 14713.168535702976">33757.20 ± 14713.17</a> | <a title="samples: 100, min: 265, max: 730, stddev: 97.39532380971892">495.47 ± 97.40</a> | <a title="samples: 4000, min: 4, max: 182, stddev: 54.82959230595704">48.67 ± 54.83</a> | <a title="samples: 10, min: 1094, max: 1966, stddev: 274.8875406416231">1508.20 ± 274.89</a> |
| `musli_packed` | <a title="samples: 500, min: 63, max: 63, stddev: 0">63.00 ± 0.00</a> | <a title="samples: 500, min: 64, max: 64, stddev: 0">64.00 ± 0.00</a> | <a title="samples: 10, min: 12152, max: 57583, stddev: 15034.268941654595">32770.30 ± 15034.27</a> | <a title="samples: 100, min: 313, max: 783, stddev: 99.04539312860544">549.99 ± 99.05</a> | <a title="samples: 4000, min: 16, max: 190, stddev: 48.47447442726962">55.68 ± 48.47</a> | <a title="samples: 10, min: 488, max: 872, stddev: 121.05139404401753">670.40 ± 121.05</a> |
| `musli_storage` | <a title="samples: 500, min: 84, max: 91, stddev: 1.280818488311287">88.25 ± 1.28</a> | <a title="samples: 500, min: 88, max: 94, stddev: 1.2251938622112004">91.33 ± 1.23</a> | <a title="samples: 10, min: 10977, max: 46389, stddev: 11613.773927539662">26517.60 ± 11613.77</a> | <a title="samples: 100, min: 247, max: 706, stddev: 96.35662924781045">474.00 ± 96.36</a> | <a title="samples: 4000, min: 2, max: 148, stddev: 44.30593878858108">38.72 ± 44.31</a> | <a title="samples: 10, min: 813, max: 1461, stddev: 204.27422744927958">1120.80 ± 204.27</a> |
| `musli_wire` | <a title="samples: 500, min: 96, max: 106, stddev: 1.7524143345681649">101.86 ± 1.75</a> | <a title="samples: 500, min: 102, max: 111, stddev: 1.7655310815729104">106.83 ± 1.77</a> | <a title="samples: 10, min: 12723, max: 54167, stddev: 13604.62029642871">30988.30 ± 13604.62</a> | <a title="samples: 100, min: 253, max: 720, stddev: 97.62660446824933">484.81 ± 97.63</a> | <a title="samples: 4000, min: 3, max: 177, stddev: 50.35150094833317">44.45 ± 50.35</a> | <a title="samples: 10, min: 933, max: 1677, stddev: 234.53707596028397">1286.40 ± 234.54</a> |
| `serde_cbor`[^i128] | <a title="samples: 500, min: 210, max: 213, stddev: 0.5346961754117986">212.69 ± 0.53</a> | <a title="samples: 500, min: 218, max: 222, stddev: 0.847610759724064">221.17 ± 0.85</a> | <a title="samples: 10, min: 19941, max: 65898, stddev: 15353.406965556536">38517.50 ± 15353.41</a> | <a title="samples: 100, min: 344, max: 806, stddev: 97.04497668607067">573.25 ± 97.04</a> | <a title="samples: 4000, min: 6, max: 251, stddev: 80.65625115110487">66.15 ± 80.66</a> | <a title="samples: 10, min: 1062, max: 1900, stddev: 264.53135919962307">1460.60 ± 264.53</a> |

#### Speedy sizes

> **Missing features:**
> - `isize` - `isize` types are not supported.
> - `cstring` - `CString`'s are not supported.

This is a test suite for speedy features.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `musli_descriptive` | <a title="samples: 500, min: 142, max: 151, stddev: 1.5066187308008552">147.31 ± 1.51</a> | <a title="samples: 500, min: 148, max: 157, stddev: 1.4568459081179361">153.36 ± 1.46</a> | <a title="samples: 10, min: 20863, max: 76189, stddev: 16011.1090671446">51615.20 ± 16011.11</a> | <a title="samples: 100, min: 399, max: 929, stddev: 115.78223697959892">649.44 ± 115.78</a> | <a title="samples: 4000, min: 4, max: 178, stddev: 61.49601393383164">51.87 ± 61.50</a> | <a title="samples: 10, min: 1094, max: 2075, stddev: 355.0456449528708">1562.70 ± 355.05</a> |
| `musli_packed` | <a title="samples: 500, min: 87, max: 87, stddev: 0">87.00 ± 0.00</a> | <a title="samples: 500, min: 88, max: 88, stddev: 0">88.00 ± 0.00</a> | <a title="samples: 10, min: 19140, max: 75044, stddev: 16137.649090248555">50607.20 ± 16137.65</a> | <a title="samples: 100, min: 455, max: 993, stddev: 118.21224090592312">707.69 ± 118.21</a> | <a title="samples: 4000, min: 16, max: 188, stddev: 51.97509226543033">58.03 ± 51.98</a> | <a title="samples: 10, min: 488, max: 920, stddev: 156.35037575906236">694.40 ± 156.35</a> |
| `musli_storage` | <a title="samples: 500, min: 113, max: 120, stddev: 1.3242356285797454">117.32 ± 1.32</a> | <a title="samples: 500, min: 115, max: 123, stddev: 1.2658135723715367">120.35 ± 1.27</a> | <a title="samples: 10, min: 17350, max: 62185, stddev: 12927.496600657068">42189.80 ± 12927.50</a> | <a title="samples: 100, min: 375, max: 896, stddev: 114.19683182995925">620.94 ± 114.20</a> | <a title="samples: 4000, min: 2, max: 146, stddev: 50.08570871765708">41.69 ± 50.09</a> | <a title="samples: 10, min: 813, max: 1542, stddev: 263.84125909341776">1161.30 ± 263.84</a> |
| `musli_wire` | <a title="samples: 500, min: 126, max: 136, stddev: 1.8188908708330995">131.81 ± 1.82</a> | <a title="samples: 500, min: 131, max: 141, stddev: 1.6698970028118476">136.96 ± 1.67</a> | <a title="samples: 10, min: 19358, max: 70864, stddev: 14885.987432481596">47974.40 ± 14885.99</a> | <a title="samples: 100, min: 387, max: 919, stddev: 116.08491504067182">637.45 ± 116.08</a> | <a title="samples: 4000, min: 3, max: 173, stddev: 56.37342208656458">47.56 ± 56.37</a> | <a title="samples: 10, min: 933, max: 1770, stddev: 302.9288530331834">1332.90 ± 302.93</a> |
| `speedy` | <a title="samples: 500, min: 87, max: 87, stddev: 0">87.00 ± 0.00</a> | <a title="samples: 500, min: 88, max: 88, stddev: 0">88.00 ± 0.00</a> | <a title="samples: 10, min: 14500, max: 49508, stddev: 9951.212014624149">33687.20 ± 9951.21</a> | <a title="samples: 100, min: 395, max: 921, stddev: 115.5135537501985">643.33 ± 115.51</a> | <a title="samples: 4000, min: 4, max: 152, stddev: 43.70155850767784">39.53 ± 43.70</a> | <a title="samples: 10, min: 484, max: 916, stddev: 156.35037575906236">690.40 ± 156.35</a> |

#### ε-serde sizes

> **Custom environment:**
> - `MUSLI_VEC_RANGE=10000..20000` - ε-serde benefits from larger inputs, this ensures that the size of the supported suite (primarily `mesh`) reflects that by making the inputs bigger.


This is a test suite for ε-serde features

Since ε-serde works best for larger inputs,
we increase the size of the input being deserialized.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `epserde` | <a title="samples: 500, min: 176, max: 176, stddev: 0">176.00 ± 0.00</a> | <a title="samples: 500, min: 176, max: 176, stddev: 0">176.00 ± 0.00</a> | - | - | - | <a title="samples: 10, min: 517888, max: 903568, stddev: 114727.41475619504">674401.60 ± 114727.41</a> |
| `musli_descriptive` | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 477189, max: 980191, stddev: 136607.76031997596">824339.60 ± 136607.76</a> | <a title="samples: 100, min: 388, max: 933, stddev: 117.03841249777783">634.90 ± 117.04</a> | <a title="samples: 4000, min: 4, max: 50173, stddev: 12482.538507407333">4670.64 ± 12482.54</a> | <a title="samples: 10, min: 1175898, max: 2051714, stddev: 260527.16207307443">1531314.50 ± 260527.16</a> |
| `musli_packed` | <a title="samples: 500, min: 95, max: 95, stddev: 0">95.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 373894, max: 635712, stddev: 80692.72130619962">561265.00 ± 80692.72</a> | <a title="samples: 100, min: 441, max: 991, stddev: 119.43036925338545">696.87 ± 119.43</a> | <a title="samples: 4000, min: 16, max: 20161, stddev: 5010.736010426201">1906.01 ± 5010.74</a> | <a title="samples: 10, min: 517832, max: 903512, stddev: 114727.41475619504">674345.60 ± 114727.41</a> |
| `musli_storage` | <a title="samples: 500, min: 122, max: 131, stddev: 1.3556681009745792">127.86 ± 1.36</a> | <a title="samples: 500, min: 127, max: 134, stddev: 1.3827783625729677">130.88 ± 1.38</a> | <a title="samples: 10, min: 311478, max: 552661, stddev: 71044.21728958662">482614.10 ± 71044.22</a> | <a title="samples: 100, min: 363, max: 902, stddev: 115.48063863695938">606.11 ± 115.48</a> | <a title="samples: 4000, min: 2, max: 20121, stddev: 5001.599059857751">1890.15 ± 5001.60</a> | <a title="samples: 10, min: 873832, max: 1524668, stddev: 193602.8367986637">1137948.90 ± 193602.84</a> |
| `musli_wire` | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 453056, max: 913257, stddev: 125780.0236707324">771645.70 ± 125780.02</a> | <a title="samples: 100, min: 377, max: 922, stddev: 116.93856335700383">621.82 ± 116.94</a> | <a title="samples: 4000, min: 3, max: 45207, stddev: 11254.26837820462">4211.44 ± 11254.27</a> | <a title="samples: 10, min: 1003289, max: 1750545, stddev: 222284.69048769417">1306534.30 ± 222284.69</a> |

#### Müsli vs zerocopy sizes

Compares [`musli-zerocopy`] with [`zerocopy`].

Note that `zerocopy` only supports packed primitives, so we're only comparing with that suite.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `musli_zerocopy` | <a title="samples: 500, min: 112, max: 112, stddev: 0">112.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - | - |
| `zerocopy` | - | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - | - |

#### Bitcode derive sizes

> **Missing features:**
> - `cstring` - `CString`'s are not supported.

Uses a custom derive-based framework which does not support everything Müsli and serde does.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `derive_bitcode` | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 106, max: 106, stddev: 0">106.00 ± 0.00</a> | <a title="samples: 10, min: 16854, max: 35937, stddev: 6106.095908188799">29711.40 ± 6106.10</a> | <a title="samples: 100, min: 358, max: 875, stddev: 108.95348135787124">610.17 ± 108.95</a> | <a title="samples: 4000, min: 1, max: 147, stddev: 47.113127416888794">39.09 ± 47.11</a> | <a title="samples: 10, min: 481, max: 913, stddev: 142.79495789417777">745.00 ± 142.79</a> |
| `musli_descriptive` | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 25295, max: 61463, stddev: 11847.805317863726">49647.50 ± 11847.81</a> | <a title="samples: 100, min: 399, max: 929, stddev: 111.46204510953493">656.45 ± 111.46</a> | <a title="samples: 4000, min: 4, max: 191, stddev: 65.10617330128954">54.13 ± 65.11</a> | <a title="samples: 10, min: 1094, max: 2075, stddev: 324.2635502180287">1693.50 ± 324.26</a> |
| `musli_packed` | <a title="samples: 500, min: 95, max: 95, stddev: 0">95.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 22958, max: 63679, stddev: 12161.381693294557">48955.90 ± 12161.38</a> | <a title="samples: 100, min: 455, max: 993, stddev: 113.68683784853901">714.73 ± 113.69</a> | <a title="samples: 4000, min: 16, max: 191, stddev: 54.262294917834616">59.73 ± 54.26</a> | <a title="samples: 10, min: 488, max: 920, stddev: 142.79495789417777">752.00 ± 142.79</a> |
| `musli_storage` | <a title="samples: 500, min: 122, max: 131, stddev: 1.3556681009745792">127.86 ± 1.36</a> | <a title="samples: 500, min: 127, max: 134, stddev: 1.3827783625729677">130.88 ± 1.38</a> | <a title="samples: 10, min: 21145, max: 49729, stddev: 9297.125779508418">40343.20 ± 9297.13</a> | <a title="samples: 100, min: 375, max: 896, stddev: 109.76911951910701">627.98 ± 109.77</a> | <a title="samples: 4000, min: 2, max: 151, stddev: 53.10567152150549">43.68 ± 53.11</a> | <a title="samples: 10, min: 813, max: 1542, stddev: 240.96649144642498">1258.50 ± 240.97</a> |
| `musli_wire` | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 23533, max: 57105, stddev: 10938.562155969128">46124.40 ± 10938.56</a> | <a title="samples: 100, min: 387, max: 919, stddev: 111.71110016466581">644.51 ± 111.71</a> | <a title="samples: 4000, min: 3, max: 179, stddev: 59.633462174667244">49.69 ± 59.63</a> | <a title="samples: 10, min: 933, max: 1770, stddev: 276.6652309199694">1444.50 ± 276.67</a> |

#### BSON sizes

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `u64` - Format is limited to the bounds of signed 64-bit integers.
> - `empty` - Empty variants are not supported.
> - `newtype` - Newtype variants are not supported.
> - `number-key` - Maps with numerical keys like `HashMap<u32, T>` are not supported.

Specific comparison to BSON, because the format is limited in capabilities.

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `bson` | <a title="samples: 500, min: 240, max: 241, stddev: 0.22992172581119874">240.94 ± 0.23</a> | <a title="samples: 500, min: 289, max: 289, stddev: 0">289.00 ± 0.00</a> | <a title="samples: 10, min: 43093, max: 106608, stddev: 21441.159101363897">82425.30 ± 21441.16</a> | <a title="samples: 100, min: 429, max: 1038, stddev: 116.21976208889778">731.37 ± 116.22</a> | <a title="samples: 2500, min: 22, max: 305, stddev: 114.65799941321195">118.24 ± 114.66</a> | <a title="samples: 10, min: 2002, max: 3269, stddev: 466.4043739074495">2436.40 ± 466.40</a> |
| `musli_descriptive` | <a title="samples: 500, min: 111, max: 118, stddev: 1.3158054567450292">115.28 ± 1.32</a> | <a title="samples: 500, min: 117, max: 124, stddev: 1.252956503634502">121.39 ± 1.25</a> | <a title="samples: 10, min: 22021, max: 56575, stddev: 11366.720621621698">43841.90 ± 11366.72</a> | <a title="samples: 100, min: 262, max: 869, stddev: 115.76712659472898">563.82 ± 115.77</a> | <a title="samples: 2500, min: 4, max: 187, stddev: 60.226860606875476">58.85 ± 60.23</a> | <a title="samples: 10, min: 1203, max: 1966, stddev: 280.8733522426077">1464.60 ± 280.87</a> |
| `musli_packed` | <a title="samples: 500, min: 63, max: 63, stddev: 0">63.00 ± 0.00</a> | <a title="samples: 500, min: 64, max: 64, stddev: 0">64.00 ± 0.00</a> | <a title="samples: 10, min: 20550, max: 59197, stddev: 12505.213420409906">44063.90 ± 12505.21</a> | <a title="samples: 100, min: 319, max: 948, stddev: 119.38412792327128">631.70 ± 119.38</a> | <a title="samples: 2500, min: 16, max: 191, stddev: 51.180023140284234">65.51 ± 51.18</a> | <a title="samples: 10, min: 536, max: 872, stddev: 123.68734777656121">651.20 ± 123.69</a> |
| `musli_storage` | <a title="samples: 500, min: 84, max: 89, stddev: 1.0394537026726993">87.24 ± 1.04</a> | <a title="samples: 500, min: 87, max: 92, stddev: 0.9957911427603747">90.38 ± 1.00</a> | <a title="samples: 10, min: 17313, max: 45421, stddev: 9094.942519884335">35094.60 ± 9094.94</a> | <a title="samples: 100, min: 244, max: 845, stddev: 114.40807488984332">543.18 ± 114.41</a> | <a title="samples: 2500, min: 2, max: 149, stddev: 46.92655737809889">44.57 ± 46.93</a> | <a title="samples: 10, min: 894, max: 1461, stddev: 208.72239937294702">1088.40 ± 208.72</a> |
| `musli_wire` | <a title="samples: 500, min: 95, max: 104, stddev: 1.6029210835221925">100.74 ± 1.60</a> | <a title="samples: 500, min: 101, max: 109, stddev: 1.4871233977044382">105.91 ± 1.49</a> | <a title="samples: 10, min: 20054, max: 52487, stddev: 10619.612921853603">40522.30 ± 10619.61</a> | <a title="samples: 100, min: 250, max: 857, stddev: 115.97885841824795">552.62 ± 115.98</a> | <a title="samples: 2500, min: 3, max: 183, stddev: 54.62712120403199">52.78 ± 54.63</a> | <a title="samples: 10, min: 1026, max: 1677, stddev: 239.64423631708735">1249.20 ± 239.64</a> |

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

| **framework** | `primitives` | `primpacked` | `large` | `allocated` | `medium_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `miniserde` | <a title="samples: 500, min: 312, max: 326, stddev: 2.2674205609017446">319.30 ± 2.27</a> | <a title="samples: 500, min: 347, max: 361, stddev: 2.460792555255309">355.35 ± 2.46</a> | <a title="samples: 10, min: 11381, max: 32089, stddev: 7047.0232304143865">20566.30 ± 7047.02</a> | <a title="samples: 100, min: 42, max: 154, stddev: 32.055788868783125">98.08 ± 32.06</a> | <a title="samples: 500, min: 7, max: 7, stddev: 0">7.00 ± 0.00</a> | <a title="samples: 10, min: 3450, max: 5975, stddev: 859.0277120093391">4874.70 ± 859.03</a> |
| `musli_json`[^incomplete] | <a title="samples: 500, min: 302, max: 317, stddev: 2.3087754329947305">310.67 ± 2.31</a> | <a title="samples: 500, min: 339, max: 353, stddev: 2.5256729796234514">346.68 ± 2.53</a> | <a title="samples: 10, min: 11086, max: 31243, stddev: 6860.941743667556">20023.70 ± 6860.94</a> | <a title="samples: 100, min: 42, max: 154, stddev: 32.055788868783125">98.08 ± 32.06</a> | <a title="samples: 500, min: 7, max: 7, stddev: 0">7.00 ± 0.00</a> | <a title="samples: 10, min: 2294, max: 4011, stddev: 577.4698260515436">3261.00 ± 577.47</a> |
| `serde_json`[^incomplete] | <a title="samples: 500, min: 302, max: 317, stddev: 2.3087754329947305">310.67 ± 2.31</a> | <a title="samples: 500, min: 339, max: 353, stddev: 2.5256729796234514">346.68 ± 2.53</a> | <a title="samples: 10, min: 11086, max: 31243, stddev: 6860.941743667556">20023.70 ± 6860.94</a> | <a title="samples: 100, min: 42, max: 154, stddev: 32.055788868783125">98.08 ± 32.06</a> | <a title="samples: 500, min: 7, max: 7, stddev: 0">7.00 ± 0.00</a> | <a title="samples: 10, min: 2294, max: 4011, stddev: 577.4698260515436">3261.00 ± 577.47</a> |


[^bson]: BSON does not support serializing directly in-place [without patches](https://github.com/mongodb/bson-rust/pull/328). As a result it is expected to be much slower.
[^i128]: Lacks 128-bit support.
[^incomplete]: These formats do not support a wide range of Rust types. Exact level of support varies. But from a size perspective it makes size comparisons either unfair or simply an esoteric exercise since they can (or cannot) make stricter assumptions as a result.
[^musli_value]: `musli-value` is a heap-allocated, in-memory format. Deserialization is expected to be as fast as a dynamic in-memory structure can be traversed, but serialization requires a lot of allocations. It is only included for reference.
[`rkyv`]: https://docs.rs/rkyv
[`zerocopy`]: https://docs.rs/zerocopy
[`musli-zerocopy`]: https://docs.rs/musli-zerocopy
