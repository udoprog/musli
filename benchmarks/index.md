# Benchmarks and size comparisons

> The following are the results of preliminary benchmarking and should be
> taken with a big grain of 🧂.

Identifiers which are used in tests:

- `dec` - Decode a type.
- `enc` - Encode a type.
- `primitives` - A small object containing one of each primitive type and a string and a byte array.
- `packed` - Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries.
- `full_enum` - An enum with every kind of supported variant.
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

**CPU:** Intel(R) Core(TM) i9-9900K CPU @ 3.60GHz 4743MHz

**Memory:** 67317MB

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
| `dec/primitives/bincode1`[^bincode1] | **134.59ns** ± 0.11ns | 134.40ns &mdash; 134.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/bincode1/report/) |
| `dec/primitives/bincode_derive` | **161.17ns** ± 0.19ns | 160.84ns &mdash; 161.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/bincode_derive/report/) |
| `dec/primitives/bincode_serde`[^bincode_serde] | **200.44ns** ± 0.20ns | 200.10ns &mdash; 200.87ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/bincode_serde/report/) |
| `dec/primitives/musli_descriptive` | **1.18μs** ± 0.78ns | 1.17μs &mdash; 1.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **22.14ns** ± 0.02ns | 22.10ns &mdash; 22.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **858.94ns** ± 1.17ns | 856.83ns &mdash; 861.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_value`[^musli_value] | **397.21ns** ± 0.43ns | 396.43ns &mdash; 398.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **865.50ns** ± 0.82ns | 864.14ns &mdash; 867.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/musli_wire/report/) |
| `dec/primitives/postcard` | **267.33ns** ± 0.18ns | 267.00ns &mdash; 267.71ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/postcard/report/) |
| `dec/primitives/serde_bincode` | **38.60ns** ± 0.25ns | 38.25ns &mdash; 38.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bincode/report/) |
| `dec/primitives/serde_bitcode` | **1.26μs** ± 2.01ns | 1.25μs &mdash; 1.26μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_bitcode/report/) |
| `dec/primitives/serde_rmp` | **321.46ns** ± 0.30ns | 320.92ns &mdash; 322.09ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_primitives/serde_rmp/report/) |

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
| `enc/primitives/bincode1`[^bincode1] | **106.21ns** ± 0.07ns | 106.08ns &mdash; 106.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/bincode1/report/) |
| `enc/primitives/bincode_derive` | **118.07ns** ± 0.48ns | 117.33ns &mdash; 119.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/bincode_derive/report/) |
| `enc/primitives/bincode_serde`[^bincode_serde] | **120.27ns** ± 0.46ns | 119.54ns &mdash; 121.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/bincode_serde/report/) |
| `enc/primitives/musli_descriptive` | **963.09ns** ± 0.96ns | 961.43ns &mdash; 965.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **19.93ns** ± 0.02ns | 19.89ns &mdash; 19.97ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **567.08ns** ± 0.64ns | 565.92ns &mdash; 568.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.15μs** ± 0.67ns | 1.15μs &mdash; 1.16μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **834.98ns** ± 1.54ns | 832.47ns &mdash; 838.39ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/musli_wire/report/) |
| `enc/primitives/postcard` | **432.13ns** ± 0.50ns | 431.26ns &mdash; 433.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/postcard/report/) |
| `enc/primitives/serde_bincode` | **31.23ns** ± 0.34ns | 30.74ns &mdash; 31.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bincode/report/) |
| `enc/primitives/serde_bitcode` | **3.69μs** ± 4.18ns | 3.68μs &mdash; 3.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_bitcode/report/) |
| `enc/primitives/serde_rmp` | **250.00ns** ± 0.30ns | 249.48ns &mdash; 250.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_primitives/serde_rmp/report/) |


<table>
<tr>
<th colspan="3">
<code>full/dec/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/packed/bincode1`[^bincode1] | **103.85ns** ± 0.08ns | 103.71ns &mdash; 104.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/bincode1/report/) |
| `dec/packed/bincode_derive` | **131.76ns** ± 0.12ns | 131.56ns &mdash; 132.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/bincode_derive/report/) |
| `dec/packed/bincode_serde`[^bincode_serde] | **166.22ns** ± 0.15ns | 165.95ns &mdash; 166.54ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/bincode_serde/report/) |
| `dec/packed/musli_descriptive` | **1.18μs** ± 0.90ns | 1.18μs &mdash; 1.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/musli_descriptive/report/) |
| `dec/packed/musli_packed` | **26.12ns** ± 0.02ns | 26.08ns &mdash; 26.16ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/musli_packed/report/) |
| `dec/packed/musli_storage` | **894.99ns** ± 0.93ns | 893.37ns &mdash; 896.99ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/musli_storage/report/) |
| `dec/packed/musli_value`[^musli_value] | **401.28ns** ± 0.45ns | 400.52ns &mdash; 402.24ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/musli_value/report/) |
| `dec/packed/musli_wire` | **893.17ns** ± 0.76ns | 891.73ns &mdash; 894.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/musli_wire/report/) |
| `dec/packed/postcard` | **267.07ns** ± 0.31ns | 266.53ns &mdash; 267.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/postcard/report/) |
| `dec/packed/serde_bitcode` | **1.50μs** ± 1.50ns | 1.49μs &mdash; 1.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/serde_bitcode/report/) |
| `dec/packed/serde_rmp` | **396.13ns** ± 0.30ns | 395.57ns &mdash; 396.76ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_packed/serde_rmp/report/) |

<table>
<tr>
<th colspan="3">
<code>full/enc/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/packed/bincode1`[^bincode1] | **122.62ns** ± 0.12ns | 122.43ns &mdash; 122.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/bincode1/report/) |
| `enc/packed/bincode_derive` | **111.89ns** ± 0.41ns | 111.22ns &mdash; 112.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/bincode_derive/report/) |
| `enc/packed/bincode_serde`[^bincode_serde] | **119.57ns** ± 0.49ns | 118.75ns &mdash; 120.65ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/bincode_serde/report/) |
| `enc/packed/musli_descriptive` | **923.26ns** ± 0.55ns | 922.26ns &mdash; 924.41ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/musli_descriptive/report/) |
| `enc/packed/musli_packed` | **21.57ns** ± 0.02ns | 21.53ns &mdash; 21.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/musli_packed/report/) |
| `enc/packed/musli_storage` | **540.40ns** ± 0.80ns | 539.03ns &mdash; 542.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/musli_storage/report/) |
| `enc/packed/musli_value`[^musli_value] | **1.59μs** ± 1.54ns | 1.59μs &mdash; 1.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/musli_value/report/) |
| `enc/packed/musli_wire` | **585.47ns** ± 0.47ns | 584.62ns &mdash; 586.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/musli_wire/report/) |
| `enc/packed/postcard` | **431.79ns** ± 0.39ns | 431.07ns &mdash; 432.60ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/postcard/report/) |
| `enc/packed/serde_bitcode` | **4.55μs** ± 5.98ns | 4.54μs &mdash; 4.57μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/serde_bitcode/report/) |
| `enc/packed/serde_rmp` | **318.48ns** ± 0.46ns | 317.64ns &mdash; 319.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_packed/serde_rmp/report/) |


<table>
<tr>
<th colspan="3">
<code>full/dec/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/full_enum/bincode1`[^bincode1] | **693.23ns** ± 0.67ns | 692.08ns &mdash; 694.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/bincode1/report/) |
| `dec/full_enum/bincode_derive` | **1.02μs** ± 1.34ns | 1.01μs &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/bincode_derive/report/) |
| `dec/full_enum/bincode_serde`[^bincode_serde] | **1.10μs** ± 1.62ns | 1.10μs &mdash; 1.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/bincode_serde/report/) |
| `dec/full_enum/musli_descriptive` | **2.49μs** ± 2.40ns | 2.49μs &mdash; 2.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/musli_descriptive/report/) |
| `dec/full_enum/musli_packed` | **513.46ns** ± 0.86ns | 511.96ns &mdash; 515.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/musli_packed/report/) |
| `dec/full_enum/musli_storage` | **1.85μs** ± 2.13ns | 1.85μs &mdash; 1.86μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/musli_storage/report/) |
| `dec/full_enum/musli_value`[^musli_value] | **981.31ns** ± 1.13ns | 979.27ns &mdash; 983.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/musli_value/report/) |
| `dec/full_enum/musli_wire` | **1.87μs** ± 1.79ns | 1.87μs &mdash; 1.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/musli_wire/report/) |
| `dec/full_enum/postcard` | **939.97ns** ± 0.84ns | 938.43ns &mdash; 941.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/postcard/report/) |
| `dec/full_enum/serde_bitcode` | **9.14μs** ± 16.77ns | 9.11μs &mdash; 9.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/serde_bitcode/report/) |
| `dec/full_enum/serde_rmp` | **2.13μs** ± 3.50ns | 2.12μs &mdash; 2.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_full_enum/serde_rmp/report/) |

<table>
<tr>
<th colspan="3">
<code>full/enc/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_full.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/full_enum/bincode1`[^bincode1] | **304.24ns** ± 0.32ns | 303.66ns &mdash; 304.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/bincode1/report/) |
| `enc/full_enum/bincode_derive` | **381.24ns** ± 8.52ns | 367.36ns &mdash; 400.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/bincode_derive/report/) |
| `enc/full_enum/bincode_serde`[^bincode_serde] | **389.36ns** ± 9.13ns | 374.44ns &mdash; 409.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/bincode_serde/report/) |
| `enc/full_enum/musli_descriptive` | **1.40μs** ± 1.82ns | 1.40μs &mdash; 1.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/musli_descriptive/report/) |
| `enc/full_enum/musli_packed` | **134.97ns** ± 0.14ns | 134.71ns &mdash; 135.27ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/musli_packed/report/) |
| `enc/full_enum/musli_storage` | **914.27ns** ± 0.93ns | 912.65ns &mdash; 916.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/musli_storage/report/) |
| `enc/full_enum/musli_value`[^musli_value] | **3.44μs** ± 3.60ns | 3.43μs &mdash; 3.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/musli_value/report/) |
| `enc/full_enum/musli_wire` | **1.39μs** ± 1.83ns | 1.38μs &mdash; 1.39μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/musli_wire/report/) |
| `enc/full_enum/postcard` | **893.97ns** ± 0.90ns | 892.30ns &mdash; 895.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/postcard/report/) |
| `enc/full_enum/serde_bitcode` | **12.25μs** ± 14.07ns | 12.22μs &mdash; 12.28μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/serde_bitcode/report/) |
| `enc/full_enum/serde_rmp` | **648.69ns** ± 0.57ns | 647.62ns &mdash; 649.87ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_full_enum/serde_rmp/report/) |


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
| `dec/large/bincode1`[^bincode1] | **52.48μs** ± 46.38ns | 52.40μs &mdash; 52.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/bincode1/report/) |
| `dec/large/bincode_derive` | **56.57μs** ± 61.69ns | 56.47μs &mdash; 56.71μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/bincode_derive/report/) |
| `dec/large/bincode_serde`[^bincode_serde] | **64.47μs** ± 49.99ns | 64.37μs &mdash; 64.57μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/bincode_serde/report/) |
| `dec/large/musli_descriptive` | **194.68μs** ± 252.48ns | 194.25μs &mdash; 195.23μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **36.88μs** ± 18.67ns | 36.85μs &mdash; 36.92μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **146.17μs** ± 176.79ns | 145.87μs &mdash; 146.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_storage/report/) |
| `dec/large/musli_value`[^musli_value] | **72.79μs** ± 101.28ns | 72.63μs &mdash; 73.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **172.44μs** ± 197.48ns | 172.14μs &mdash; 172.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/musli_wire/report/) |
| `dec/large/postcard` | **71.25μs** ± 69.73ns | 71.14μs &mdash; 71.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/postcard/report/) |
| `dec/large/serde_bincode` | **13.45μs** ± 20.38ns | 13.42μs &mdash; 13.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bincode/report/) |
| `dec/large/serde_bitcode` | **81.76μs** ± 92.60ns | 81.59μs &mdash; 81.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_bitcode/report/) |
| `dec/large/serde_rmp` | **120.33μs** ± 116.69ns | 120.12μs &mdash; 120.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_large/serde_rmp/report/) |

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
| `enc/large/bincode1`[^bincode1] | **23.38μs** ± 21.66ns | 23.34μs &mdash; 23.42μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/bincode1/report/) |
| `enc/large/bincode_derive` | **18.95μs** ± 69.88ns | 18.84μs &mdash; 19.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/bincode_derive/report/) |
| `enc/large/bincode_serde`[^bincode_serde] | **19.91μs** ± 87.60ns | 19.77μs &mdash; 20.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/bincode_serde/report/) |
| `enc/large/musli_descriptive` | **102.64μs** ± 101.57ns | 102.47μs &mdash; 102.87μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **10.38μs** ± 6.50ns | 10.37μs &mdash; 10.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **68.19μs** ± 60.77ns | 68.08μs &mdash; 68.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_storage/report/) |
| `enc/large/musli_value`[^musli_value] | **304.98μs** ± 556.16ns | 304.07μs &mdash; 306.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **105.11μs** ± 69.98ns | 104.98μs &mdash; 105.26μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/musli_wire/report/) |
| `enc/large/postcard` | **72.14μs** ± 86.12ns | 71.99μs &mdash; 72.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/postcard/report/) |
| `enc/large/serde_bincode` | **4.88μs** ± 14.98ns | 4.86μs &mdash; 4.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bincode/report/) |
| `enc/large/serde_bitcode` | **94.91μs** ± 90.42ns | 94.74μs &mdash; 95.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_bitcode/report/) |
| `enc/large/serde_rmp` | **62.53μs** ± 75.76ns | 62.39μs &mdash; 62.69μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_large/serde_rmp/report/) |


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
| `dec/allocated/bincode1`[^bincode1] | **3.04μs** ± 2.29ns | 3.04μs &mdash; 3.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/bincode1/report/) |
| `dec/allocated/bincode_derive` | **3.75μs** ± 5.65ns | 3.74μs &mdash; 3.77μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/bincode_derive/report/) |
| `dec/allocated/bincode_serde`[^bincode_serde] | **4.12μs** ± 4.58ns | 4.11μs &mdash; 4.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/bincode_serde/report/) |
| `dec/allocated/musli_descriptive` | **3.39μs** ± 4.27ns | 3.39μs &mdash; 3.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **2.01μs** ± 1.97ns | 2.00μs &mdash; 2.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **2.90μs** ± 3.02ns | 2.89μs &mdash; 2.90μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_value`[^musli_value] | **1.83μs** ± 1.66ns | 1.82μs &mdash; 1.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **3.11μs** ± 3.05ns | 3.10μs &mdash; 3.11μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/musli_wire/report/) |
| `dec/allocated/postcard` | **3.21μs** ± 2.58ns | 3.21μs &mdash; 3.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/postcard/report/) |
| `dec/allocated/serde_bincode` | **899.32ns** ± 0.77ns | 898.22ns &mdash; 900.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bincode/report/) |
| `dec/allocated/serde_bitcode` | **5.76μs** ± 8.08ns | 5.74μs &mdash; 5.77μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_bitcode/report/) |
| `dec/allocated/serde_rmp` | **4.02μs** ± 6.82ns | 4.01μs &mdash; 4.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_allocated/serde_rmp/report/) |

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
| `enc/allocated/bincode1`[^bincode1] | **385.40ns** ± 0.30ns | 384.85ns &mdash; 386.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/bincode1/report/) |
| `enc/allocated/bincode_derive` | **437.42ns** ± 1.68ns | 434.68ns &mdash; 441.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/bincode_derive/report/) |
| `enc/allocated/bincode_serde`[^bincode_serde] | **448.11ns** ± 1.96ns | 445.11ns &mdash; 452.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/bincode_serde/report/) |
| `enc/allocated/musli_descriptive` | **589.85ns** ± 0.69ns | 588.61ns &mdash; 591.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **177.91ns** ± 0.19ns | 177.55ns &mdash; 178.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **526.65ns** ± 0.79ns | 525.22ns &mdash; 528.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_value`[^musli_value] | **2.58μs** ± 3.52ns | 2.58μs &mdash; 2.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **976.63ns** ± 1.20ns | 974.50ns &mdash; 979.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/musli_wire/report/) |
| `enc/allocated/postcard` | **1.21μs** ± 1.72ns | 1.21μs &mdash; 1.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/postcard/report/) |
| `enc/allocated/serde_bincode` | **96.63ns** ± 0.38ns | 96.09ns &mdash; 97.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bincode/report/) |
| `enc/allocated/serde_bitcode` | **7.68μs** ± 8.62ns | 7.67μs &mdash; 7.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_bitcode/report/) |
| `enc/allocated/serde_rmp` | **739.90ns** ± 0.73ns | 738.69ns &mdash; 741.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_allocated/serde_rmp/report/) |


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
| `dec/mesh/bincode1`[^bincode1] | **532.48ns** ± 0.51ns | 531.58ns &mdash; 533.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/bincode1/report/) |
| `dec/mesh/bincode_serde`[^bincode_serde] | **409.96ns** ± 0.45ns | 409.17ns &mdash; 410.93ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/bincode_serde/report/) |
| `dec/mesh/musli_descriptive` | **8.13μs** ± 11.35ns | 8.11μs &mdash; 8.16μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **86.49ns** ± 0.07ns | 86.36ns &mdash; 86.64ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **4.98μs** ± 5.77ns | 4.97μs &mdash; 4.99μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_value`[^musli_value] | **2.26μs** ± 2.46ns | 2.26μs &mdash; 2.27μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/musli_value/report/) |
| `dec/mesh/musli_wire` | **5.70μs** ± 6.24ns | 5.69μs &mdash; 5.71μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/musli_wire/report/) |
| `dec/mesh/postcard` | **408.36ns** ± 0.38ns | 407.69ns &mdash; 409.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/postcard/report/) |
| `dec/mesh/serde_bincode` | **505.72ns** ± 4.05ns | 499.99ns &mdash; 511.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/serde_bincode/report/) |
| `dec/mesh/serde_bitcode` | **3.57μs** ± 34.68ns | 3.50μs &mdash; 3.64μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/serde_bitcode/report/) |
| `dec/mesh/serde_rmp` | **2.88μs** ± 2.04ns | 2.88μs &mdash; 2.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/dec_mesh/serde_rmp/report/) |

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
| `enc/mesh/bincode1`[^bincode1] | **700.16ns** ± 0.48ns | 699.27ns &mdash; 701.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/bincode1/report/) |
| `enc/mesh/bincode_serde`[^bincode_serde] | **225.92ns** ± 0.92ns | 224.53ns &mdash; 228.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/bincode_serde/report/) |
| `enc/mesh/musli_descriptive` | **3.43μs** ± 3.28ns | 3.43μs &mdash; 3.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **33.67ns** ± 0.03ns | 33.62ns &mdash; 33.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **2.55μs** ± 2.82ns | 2.55μs &mdash; 2.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_value`[^musli_value] | **17.44μs** ± 25.46ns | 17.40μs &mdash; 17.50μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/musli_value/report/) |
| `enc/mesh/musli_wire` | **4.72μs** ± 4.05ns | 4.71μs &mdash; 4.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/musli_wire/report/) |
| `enc/mesh/postcard` | **385.83ns** ± 0.45ns | 385.05ns &mdash; 386.79ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/postcard/report/) |
| `enc/mesh/serde_bincode` | **309.01ns** ± 0.16ns | 308.79ns &mdash; 309.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/serde_bincode/report/) |
| `enc/mesh/serde_bitcode` | **4.72μs** ± 5.57ns | 4.71μs &mdash; 4.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/serde_bitcode/report/) |
| `enc/mesh/serde_rmp` | **1.59μs** ± 1.66ns | 1.58μs &mdash; 1.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-full/enc_mesh/serde_rmp/report/) |



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
| `dec/primitives/musli_json` | **3.61μs** ± 3.00ns | 3.61μs &mdash; 3.62μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **4.45μs** ± 4.12ns | 4.44μs &mdash; 4.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_primitives/serde_json/report/) |

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
| `enc/primitives/musli_json` | **1.39μs** ± 1.31ns | 1.38μs &mdash; 1.39μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **1.31μs** ± 1.11ns | 1.30μs &mdash; 1.31μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_primitives/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>text/dec/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/dec_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/packed/musli_json` | **4.19μs** ± 4.35ns | 4.18μs &mdash; 4.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_packed/musli_json/report/) |
| `dec/packed/serde_json` | **4.59μs** ± 4.04ns | 4.58μs &mdash; 4.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_packed/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>text/enc/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/enc_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/packed/musli_json` | **1.20μs** ± 0.69ns | 1.19μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_packed/musli_json/report/) |
| `enc/packed/serde_json` | **1.37μs** ± 1.46ns | 1.37μs &mdash; 1.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_packed/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>text/dec/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/dec_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/full_enum/musli_json` | **8.19μs** ± 23.00ns | 8.15μs &mdash; 8.24μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_full_enum/musli_json/report/) |
| `dec/full_enum/serde_json` | **7.67μs** ± 5.78ns | 7.66μs &mdash; 7.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_full_enum/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>text/enc/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-text/enc_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_text.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/full_enum/musli_json` | **2.61μs** ± 2.46ns | 2.60μs &mdash; 2.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_full_enum/musli_json/report/) |
| `enc/full_enum/serde_json` | **2.35μs** ± 2.43ns | 2.34μs &mdash; 2.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_full_enum/serde_json/report/) |


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
| `dec/large/musli_json` | **612.88μs** ± 829.36ns | 611.42μs &mdash; 614.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **566.20μs** ± 525.69ns | 565.36μs &mdash; 567.38μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_large/serde_json/report/) |

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
| `enc/large/musli_json` | **196.03μs** ± 182.78ns | 195.70μs &mdash; 196.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **184.23μs** ± 135.58ns | 184.01μs &mdash; 184.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_large/serde_json/report/) |


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
| `dec/allocated/musli_json` | **9.82μs** ± 18.88ns | 9.79μs &mdash; 9.86μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **7.84μs** ± 14.13ns | 7.81μs &mdash; 7.87μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_allocated/serde_json/report/) |

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
| `enc/allocated/musli_json` | **2.47μs** ± 2.29ns | 2.46μs &mdash; 2.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **2.46μs** ± 2.14ns | 2.46μs &mdash; 2.47μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_allocated/serde_json/report/) |


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
| `dec/mesh/musli_json` | **29.67μs** ± 24.10ns | 29.63μs &mdash; 29.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_mesh/musli_json/report/) |
| `dec/mesh/serde_json` | **24.78μs** ± 20.46ns | 24.74μs &mdash; 24.82μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/dec_mesh/serde_json/report/) |

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
| `enc/mesh/musli_json` | **12.68μs** ± 12.51ns | 12.65μs &mdash; 12.70μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_mesh/musli_json/report/) |
| `enc/mesh/serde_json` | **13.74μs** ± 13.65ns | 13.72μs &mdash; 13.77μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-text/enc_mesh/serde_json/report/) |



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
| `dec/primitives/musli_descriptive` | **891.60ns** ± 1.05ns | 889.67ns &mdash; 893.79ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **15.12ns** ± 0.01ns | 15.10ns &mdash; 15.14ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **629.01ns** ± 0.62ns | 627.87ns &mdash; 630.28ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_value`[^musli_value] | **341.99ns** ± 0.47ns | 341.17ns &mdash; 342.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **610.08ns** ± 0.58ns | 608.93ns &mdash; 611.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/musli_wire/report/) |
| `dec/primitives/serde_cbor` | **1.65μs** ± 1.97ns | 1.65μs &mdash; 1.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_primitives/serde_cbor/report/) |

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
| `enc/primitives/musli_descriptive` | **328.41ns** ± 0.52ns | 327.62ns &mdash; 329.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **17.87ns** ± 0.02ns | 17.83ns &mdash; 17.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **256.06ns** ± 0.28ns | 255.58ns &mdash; 256.66ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.13μs** ± 1.30ns | 1.13μs &mdash; 1.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **495.87ns** ± 1.02ns | 493.91ns &mdash; 497.89ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/musli_wire/report/) |
| `enc/primitives/serde_cbor` | **435.00ns** ± 0.25ns | 434.52ns &mdash; 435.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_primitives/serde_cbor/report/) |


<table>
<tr>
<th colspan="3">
<code>fewer/dec/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/packed/musli_descriptive` | **935.47ns** ± 1.06ns | 933.55ns &mdash; 937.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_packed/musli_descriptive/report/) |
| `dec/packed/musli_packed` | **22.15ns** ± 0.02ns | 22.12ns &mdash; 22.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_packed/musli_packed/report/) |
| `dec/packed/musli_storage` | **642.98ns** ± 0.78ns | 641.55ns &mdash; 644.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_packed/musli_storage/report/) |
| `dec/packed/musli_value`[^musli_value] | **347.32ns** ± 0.34ns | 346.72ns &mdash; 348.04ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_packed/musli_value/report/) |
| `dec/packed/musli_wire` | **614.45ns** ± 0.34ns | 613.85ns &mdash; 615.18ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_packed/musli_wire/report/) |
| `dec/packed/serde_cbor` | **1.82μs** ± 2.19ns | 1.82μs &mdash; 1.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_packed/serde_cbor/report/) |

<table>
<tr>
<th colspan="3">
<code>fewer/enc/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/packed/musli_descriptive` | **350.50ns** ± 0.39ns | 349.82ns &mdash; 351.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_packed/musli_descriptive/report/) |
| `enc/packed/musli_packed` | **19.52ns** ± 0.02ns | 19.48ns &mdash; 19.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_packed/musli_packed/report/) |
| `enc/packed/musli_storage` | **261.81ns** ± 0.28ns | 261.32ns &mdash; 262.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_packed/musli_storage/report/) |
| `enc/packed/musli_value`[^musli_value] | **1.29μs** ± 1.47ns | 1.29μs &mdash; 1.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_packed/musli_value/report/) |
| `enc/packed/musli_wire` | **321.31ns** ± 0.37ns | 320.73ns &mdash; 322.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_packed/musli_wire/report/) |
| `enc/packed/serde_cbor` | **492.77ns** ± 0.33ns | 492.16ns &mdash; 493.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_packed/serde_cbor/report/) |


<table>
<tr>
<th colspan="3">
<code>fewer/dec/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/full_enum/musli_descriptive` | **2.11μs** ± 2.34ns | 2.11μs &mdash; 2.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_full_enum/musli_descriptive/report/) |
| `dec/full_enum/musli_packed` | **453.07ns** ± 0.63ns | 451.96ns &mdash; 454.42ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_full_enum/musli_packed/report/) |
| `dec/full_enum/musli_storage` | **1.46μs** ± 1.38ns | 1.46μs &mdash; 1.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_full_enum/musli_storage/report/) |
| `dec/full_enum/musli_value`[^musli_value] | **934.73ns** ± 1.03ns | 932.94ns &mdash; 936.97ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_full_enum/musli_value/report/) |
| `dec/full_enum/musli_wire` | **1.61μs** ± 1.89ns | 1.61μs &mdash; 1.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_full_enum/musli_wire/report/) |
| `dec/full_enum/serde_cbor` | **4.44μs** ± 3.65ns | 4.44μs &mdash; 4.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_full_enum/serde_cbor/report/) |

<table>
<tr>
<th colspan="3">
<code>fewer/enc/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_fewer.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/full_enum/musli_descriptive` | **836.81ns** ± 1.32ns | 834.43ns &mdash; 839.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_full_enum/musli_descriptive/report/) |
| `enc/full_enum/musli_packed` | **136.38ns** ± 0.21ns | 136.01ns &mdash; 136.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_full_enum/musli_packed/report/) |
| `enc/full_enum/musli_storage` | **608.61ns** ± 0.52ns | 607.72ns &mdash; 609.74ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_full_enum/musli_storage/report/) |
| `enc/full_enum/musli_value`[^musli_value] | **3.25μs** ± 2.73ns | 3.24μs &mdash; 3.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_full_enum/musli_value/report/) |
| `enc/full_enum/musli_wire` | **1.09μs** ± 1.47ns | 1.08μs &mdash; 1.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_full_enum/musli_wire/report/) |
| `enc/full_enum/serde_cbor` | **1.01μs** ± 1.02ns | 1.01μs &mdash; 1.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_full_enum/serde_cbor/report/) |


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
| `dec/large/musli_descriptive` | **140.04μs** ± 113.42ns | 139.86μs &mdash; 140.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **14.40μs** ± 13.44ns | 14.38μs &mdash; 14.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **100.36μs** ± 68.80ns | 100.24μs &mdash; 100.51μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_storage/report/) |
| `dec/large/musli_value`[^musli_value] | **52.95μs** ± 59.68ns | 52.84μs &mdash; 53.07μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **119.76μs** ± 161.82ns | 119.47μs &mdash; 120.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/musli_wire/report/) |
| `dec/large/serde_cbor` | **257.70μs** ± 273.26ns | 257.23μs &mdash; 258.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_large/serde_cbor/report/) |

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
| `enc/large/musli_descriptive` | **52.55μs** ± 58.17ns | 52.44μs &mdash; 52.67μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **5.71μs** ± 4.45ns | 5.70μs &mdash; 5.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **40.73μs** ± 40.19ns | 40.66μs &mdash; 40.81μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_storage/report/) |
| `enc/large/musli_value`[^musli_value] | **239.62μs** ± 254.13ns | 239.19μs &mdash; 240.17μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **74.71μs** ± 93.54ns | 74.55μs &mdash; 74.92μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/musli_wire/report/) |
| `enc/large/serde_cbor` | **67.74μs** ± 51.94ns | 67.65μs &mdash; 67.85μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_large/serde_cbor/report/) |


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
| `dec/allocated/musli_descriptive` | **2.35μs** ± 1.81ns | 2.35μs &mdash; 2.36μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **1.56μs** ± 2.41ns | 1.56μs &mdash; 1.57μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **2.12μs** ± 2.36ns | 2.11μs &mdash; 2.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_value`[^musli_value] | **1.30μs** ± 1.45ns | 1.30μs &mdash; 1.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **2.19μs** ± 2.11ns | 2.19μs &mdash; 2.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/musli_wire/report/) |
| `dec/allocated/serde_cbor` | **4.04μs** ± 4.54ns | 4.03μs &mdash; 4.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_allocated/serde_cbor/report/) |

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
| `enc/allocated/musli_descriptive` | **408.33ns** ± 0.41ns | 407.56ns &mdash; 409.19ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **132.83ns** ± 0.09ns | 132.65ns &mdash; 133.02ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **343.44ns** ± 0.34ns | 342.83ns &mdash; 344.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_value`[^musli_value] | **2.02μs** ± 1.90ns | 2.02μs &mdash; 2.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **385.54ns** ± 0.52ns | 384.69ns &mdash; 386.68ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/musli_wire/report/) |
| `enc/allocated/serde_cbor` | **624.06ns** ± 0.65ns | 622.87ns &mdash; 625.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_allocated/serde_cbor/report/) |


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
| `dec/mesh/musli_descriptive` | **8.21μs** ± 9.04ns | 8.19μs &mdash; 8.23μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **85.05ns** ± 0.07ns | 84.91ns &mdash; 85.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **4.98μs** ± 5.40ns | 4.97μs &mdash; 4.99μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_value`[^musli_value] | **2.29μs** ± 2.64ns | 2.28μs &mdash; 2.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/musli_value/report/) |
| `dec/mesh/musli_wire` | **5.68μs** ± 3.51ns | 5.67μs &mdash; 5.68μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/musli_wire/report/) |
| `dec/mesh/serde_cbor` | **12.78μs** ± 9.95ns | 12.76μs &mdash; 12.80μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/dec_mesh/serde_cbor/report/) |

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
| `enc/mesh/musli_descriptive` | **3.35μs** ± 3.60ns | 3.35μs &mdash; 3.36μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **33.77ns** ± 0.03ns | 33.72ns &mdash; 33.83ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **2.54μs** ± 2.37ns | 2.53μs &mdash; 2.54μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_value`[^musli_value] | **17.15μs** ± 14.37ns | 17.12μs &mdash; 17.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/musli_value/report/) |
| `enc/mesh/musli_wire` | **4.73μs** ± 5.14ns | 4.72μs &mdash; 4.74μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/musli_wire/report/) |
| `enc/mesh/serde_cbor` | **6.51μs** ± 8.32ns | 6.50μs &mdash; 6.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-fewer/enc_mesh/serde_cbor/report/) |



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
| `dec/primitives/musli_descriptive` | **1.03μs** ± 1.05ns | 1.03μs &mdash; 1.03μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **22.58ns** ± 0.02ns | 22.55ns &mdash; 22.61ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **780.84ns** ± 0.77ns | 779.56ns &mdash; 782.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_value`[^musli_value] | **370.94ns** ± 0.37ns | 370.31ns &mdash; 371.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **806.52ns** ± 0.69ns | 805.27ns &mdash; 807.95ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/musli_wire/report/) |
| `dec/primitives/speedy` | **19.23ns** ± 0.01ns | 19.21ns &mdash; 19.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_primitives/speedy/report/) |

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
| `enc/primitives/musli_descriptive` | **893.18ns** ± 1.17ns | 891.07ns &mdash; 895.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **18.75ns** ± 0.02ns | 18.72ns &mdash; 18.79ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **541.66ns** ± 0.67ns | 540.43ns &mdash; 543.05ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.20μs** ± 1.06ns | 1.20μs &mdash; 1.20μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **707.51ns** ± 1.35ns | 705.51ns &mdash; 710.60ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/musli_wire/report/) |
| `enc/primitives/speedy` | **15.93ns** ± 0.01ns | 15.91ns &mdash; 15.96ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_primitives/speedy/report/) |


<table>
<tr>
<th colspan="3">
<code>speedy/dec/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/packed/musli_descriptive` | **1.06μs** ± 0.83ns | 1.06μs &mdash; 1.06μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_packed/musli_descriptive/report/) |
| `dec/packed/musli_packed` | **23.47ns** ± 0.02ns | 23.45ns &mdash; 23.51ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_packed/musli_packed/report/) |
| `dec/packed/musli_storage` | **835.03ns** ± 1.06ns | 833.18ns &mdash; 837.29ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_packed/musli_storage/report/) |
| `dec/packed/musli_value`[^musli_value] | **366.06ns** ± 0.27ns | 365.58ns &mdash; 366.64ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_packed/musli_value/report/) |
| `dec/packed/musli_wire` | **850.00ns** ± 0.76ns | 848.63ns &mdash; 851.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_packed/musli_wire/report/) |
| `dec/packed/speedy` | **20.71ns** ± 0.02ns | 20.68ns &mdash; 20.76ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_packed/speedy/report/) |

<table>
<tr>
<th colspan="3">
<code>speedy/enc/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/packed/musli_descriptive` | **907.64ns** ± 0.98ns | 905.93ns &mdash; 909.76ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_packed/musli_descriptive/report/) |
| `enc/packed/musli_packed` | **20.35ns** ± 0.01ns | 20.32ns &mdash; 20.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_packed/musli_packed/report/) |
| `enc/packed/musli_storage` | **531.18ns** ± 0.54ns | 530.26ns &mdash; 532.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_packed/musli_storage/report/) |
| `enc/packed/musli_value`[^musli_value] | **1.36μs** ± 1.28ns | 1.36μs &mdash; 1.36μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_packed/musli_value/report/) |
| `enc/packed/musli_wire` | **582.63ns** ± 0.51ns | 581.67ns &mdash; 583.68ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_packed/musli_wire/report/) |
| `enc/packed/speedy` | **16.32ns** ± 0.02ns | 16.29ns &mdash; 16.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_packed/speedy/report/) |


<table>
<tr>
<th colspan="3">
<code>speedy/dec/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/full_enum/musli_descriptive` | **2.58μs** ± 2.29ns | 2.58μs &mdash; 2.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_full_enum/musli_descriptive/report/) |
| `dec/full_enum/musli_packed` | **744.33ns** ± 0.93ns | 742.52ns &mdash; 746.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_full_enum/musli_packed/report/) |
| `dec/full_enum/musli_storage` | **1.95μs** ± 2.39ns | 1.95μs &mdash; 1.96μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_full_enum/musli_storage/report/) |
| `dec/full_enum/musli_value`[^musli_value] | **1.02μs** ± 0.78ns | 1.02μs &mdash; 1.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_full_enum/musli_value/report/) |
| `dec/full_enum/musli_wire` | **2.12μs** ± 1.84ns | 2.11μs &mdash; 2.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_full_enum/musli_wire/report/) |
| `dec/full_enum/speedy` | **759.05ns** ± 0.94ns | 757.36ns &mdash; 761.03ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_full_enum/speedy/report/) |

<table>
<tr>
<th colspan="3">
<code>speedy/enc/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_speedy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_speedy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/full_enum/musli_descriptive` | **1.43μs** ± 2.21ns | 1.42μs &mdash; 1.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_full_enum/musli_descriptive/report/) |
| `enc/full_enum/musli_packed` | **140.26ns** ± 0.14ns | 140.01ns &mdash; 140.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_full_enum/musli_packed/report/) |
| `enc/full_enum/musli_storage` | **916.31ns** ± 0.96ns | 914.66ns &mdash; 918.40ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_full_enum/musli_storage/report/) |
| `enc/full_enum/musli_value`[^musli_value] | **3.59μs** ± 3.46ns | 3.58μs &mdash; 3.60μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_full_enum/musli_value/report/) |
| `enc/full_enum/musli_wire` | **1.28μs** ± 1.25ns | 1.28μs &mdash; 1.29μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_full_enum/musli_wire/report/) |
| `enc/full_enum/speedy` | **309.94ns** ± 0.28ns | 309.40ns &mdash; 310.50ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_full_enum/speedy/report/) |


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
| `dec/large/musli_descriptive` | **199.44μs** ± 192.06ns | 199.11μs &mdash; 199.86μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **41.02μs** ± 30.79ns | 40.96μs &mdash; 41.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **158.93μs** ± 290.34ns | 158.47μs &mdash; 159.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_storage/report/) |
| `dec/large/musli_value`[^musli_value] | **77.52μs** ± 69.12ns | 77.39μs &mdash; 77.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_value/report/) |
| `dec/large/musli_wire` | **180.74μs** ± 144.75ns | 180.48μs &mdash; 181.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/musli_wire/report/) |
| `dec/large/speedy` | **43.91μs** ± 49.51ns | 43.82μs &mdash; 44.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_large/speedy/report/) |

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
| `enc/large/musli_descriptive` | **116.83μs** ± 82.41ns | 116.69μs &mdash; 117.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **10.04μs** ± 10.61ns | 10.02μs &mdash; 10.07μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **76.68μs** ± 84.30ns | 76.53μs &mdash; 76.86μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_storage/report/) |
| `enc/large/musli_value`[^musli_value] | **297.06μs** ± 365.55ns | 296.44μs &mdash; 297.85μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_value/report/) |
| `enc/large/musli_wire` | **105.44μs** ± 85.98ns | 105.30μs &mdash; 105.63μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/musli_wire/report/) |
| `enc/large/speedy` | **10.19μs** ± 13.99ns | 10.17μs &mdash; 10.22μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_large/speedy/report/) |


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
| `dec/allocated/musli_descriptive` | **4.18μs** ± 2.89ns | 4.18μs &mdash; 4.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **2.55μs** ± 2.81ns | 2.54μs &mdash; 2.55μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **3.55μs** ± 4.84ns | 3.54μs &mdash; 3.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_value`[^musli_value] | **1.88μs** ± 1.58ns | 1.88μs &mdash; 1.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_value/report/) |
| `dec/allocated/musli_wire` | **3.44μs** ± 3.76ns | 3.43μs &mdash; 3.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/musli_wire/report/) |
| `dec/allocated/speedy` | **3.40μs** ± 2.75ns | 3.39μs &mdash; 3.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_allocated/speedy/report/) |

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
| `enc/allocated/musli_descriptive` | **635.87ns** ± 0.60ns | 634.90ns &mdash; 637.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **188.96ns** ± 0.12ns | 188.74ns &mdash; 189.23ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **576.68ns** ± 0.60ns | 575.65ns &mdash; 577.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_value`[^musli_value] | **2.55μs** ± 3.03ns | 2.54μs &mdash; 2.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_value/report/) |
| `enc/allocated/musli_wire` | **624.50ns** ± 0.84ns | 623.01ns &mdash; 626.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/musli_wire/report/) |
| `enc/allocated/speedy` | **506.55ns** ± 0.31ns | 506.00ns &mdash; 507.20ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_allocated/speedy/report/) |


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
| `dec/mesh/musli_descriptive` | **10.86μs** ± 15.04ns | 10.83μs &mdash; 10.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **92.23ns** ± 0.07ns | 92.09ns &mdash; 92.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **7.42μs** ± 7.55ns | 7.41μs &mdash; 7.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_value`[^musli_value] | **3.43μs** ± 2.81ns | 3.43μs &mdash; 3.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/musli_value/report/) |
| `dec/mesh/musli_wire` | **8.50μs** ± 9.42ns | 8.48μs &mdash; 8.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/musli_wire/report/) |
| `dec/mesh/speedy` | **73.65ns** ± 0.08ns | 73.51ns &mdash; 73.81ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/dec_mesh/speedy/report/) |

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
| `enc/mesh/musli_descriptive` | **5.04μs** ± 4.83ns | 5.03μs &mdash; 5.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **42.44ns** ± 0.03ns | 42.39ns &mdash; 42.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **4.13μs** ± 2.29ns | 4.13μs &mdash; 4.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_value`[^musli_value] | **26.29μs** ± 40.76ns | 26.21μs &mdash; 26.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/musli_value/report/) |
| `enc/mesh/musli_wire` | **5.82μs** ± 4.22ns | 5.81μs &mdash; 5.83μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/musli_wire/report/) |
| `enc/mesh/speedy` | **69.80ns** ± 0.05ns | 69.72ns &mdash; 69.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-speedy/enc_mesh/speedy/report/) |



### ε-serde

> **Custom environment:**
> - `MUSLI_VEC_RANGE=10000..20000` - ε-serde benefits from larger inputs, this ensures that the size of the supported suite (primarily `mesh`) reflects that by making the inputs bigger.


This is a test suite for ε-serde features.

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
| `dec/primitives/epserde` | **2.00μs** ± 1.82ns | 2.00μs &mdash; 2.01μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/epserde/report/) |
| `dec/primitives/musli_descriptive` | **1.16μs** ± 1.15ns | 1.16μs &mdash; 1.17μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **24.78ns** ± 0.02ns | 24.75ns &mdash; 24.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **868.15ns** ± 1.15ns | 866.06ns &mdash; 870.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_value`[^musli_value] | **397.47ns** ± 0.27ns | 397.00ns &mdash; 398.04ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/musli_value/report/) |
| `dec/primitives/musli_wire` | **866.28ns** ± 0.66ns | 865.22ns &mdash; 867.76ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/epserde` | **1.78μs** ± 2.06ns | 1.77μs &mdash; 1.78μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/epserde/report/) |
| `enc/primitives/musli_descriptive` | **966.69ns** ± 0.99ns | 965.02ns &mdash; 968.85ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **20.19ns** ± 0.01ns | 20.17ns &mdash; 20.22ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **565.79ns** ± 0.53ns | 564.81ns &mdash; 566.88ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_value`[^musli_value] | **1.24μs** ± 1.48ns | 1.23μs &mdash; 1.24μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/musli_value/report/) |
| `enc/primitives/musli_wire` | **835.83ns** ± 1.10ns | 833.87ns &mdash; 838.16ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_primitives/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>epserde/dec/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/packed/epserde` | **2.35μs** ± 2.18ns | 2.35μs &mdash; 2.36μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_packed/epserde/report/) |
| `dec/packed/musli_descriptive` | **1.18μs** ± 1.33ns | 1.18μs &mdash; 1.18μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_packed/musli_descriptive/report/) |
| `dec/packed/musli_packed` | **5.16ns** ± 0.00ns | 5.16ns &mdash; 5.17ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_packed/musli_packed/report/) |
| `dec/packed/musli_storage` | **897.30ns** ± 0.99ns | 895.52ns &mdash; 899.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_packed/musli_storage/report/) |
| `dec/packed/musli_value`[^musli_value] | **402.31ns** ± 0.46ns | 401.51ns &mdash; 403.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_packed/musli_value/report/) |
| `dec/packed/musli_wire` | **895.99ns** ± 0.82ns | 894.53ns &mdash; 897.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_packed/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>epserde/enc/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_epserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_epserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/packed/epserde` | **2.10μs** ± 1.67ns | 2.09μs &mdash; 2.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_packed/epserde/report/) |
| `enc/packed/musli_descriptive` | **922.34ns** ± 0.65ns | 921.15ns &mdash; 923.71ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_packed/musli_descriptive/report/) |
| `enc/packed/musli_packed` | **6.86ns** ± 0.00ns | 6.85ns &mdash; 6.87ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_packed/musli_packed/report/) |
| `enc/packed/musli_storage` | **539.42ns** ± 0.45ns | 538.60ns &mdash; 540.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_packed/musli_storage/report/) |
| `enc/packed/musli_value`[^musli_value] | **1.65μs** ± 2.21ns | 1.65μs &mdash; 1.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_packed/musli_value/report/) |
| `enc/packed/musli_wire` | **580.23ns** ± 0.53ns | 579.29ns &mdash; 581.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_packed/musli_wire/report/) |


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
| `dec/mesh/epserde` | **2.35μs** ± 3.24ns | 2.35μs &mdash; 2.36μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/epserde/report/) |
| `dec/mesh/musli_descriptive` | **12.70ms** ± 8.06μs | 12.69ms &mdash; 12.72ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **385.70μs** ± 599.97ns | 384.68μs &mdash; 387.02μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **7.77ms** ± 2.64μs | 7.77ms &mdash; 7.78ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_value`[^musli_value] | **5.47ms** ± 2.52μs | 5.47ms &mdash; 5.48ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/musli_value/report/) |
| `dec/mesh/musli_wire` | **8.88ms** ± 3.64μs | 8.88ms &mdash; 8.89ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/dec_mesh/musli_wire/report/) |

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
| `enc/mesh/epserde` | **113.26μs** ± 495.97ns | 112.65μs &mdash; 114.34μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/epserde/report/) |
| `enc/mesh/musli_descriptive` | **5.37ms** ± 4.12μs | 5.37ms &mdash; 5.38ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **110.06μs** ± 134.62ns | 109.81μs &mdash; 110.34μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **4.02ms** ± 2.48μs | 4.02ms &mdash; 4.03ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_value`[^musli_value] | **27.11ms** ± 38.20μs | 27.04ms &mdash; 27.19ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/musli_value/report/) |
| `enc/mesh/musli_wire` | **7.51ms** ± 3.37μs | 7.50ms &mdash; 7.52ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-epserde/enc_mesh/musli_wire/report/) |



### Müsli vs zerocopy

Compares [`musli-zerocopy`] with [`zerocopy`].

Note that `zerocopy` only supports packed primitives, so we're only comparing with that suite.

**More:**

* [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/report/)
* [Sizes](#müsli-vs-zerocopy-sizes)

<table>
<tr>
<th colspan="3">
<code>zerocopy-zerocopy/dec/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_zerocopy-zerocopy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_zerocopy-zerocopy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/packed/musli_zerocopy` | **2.66ns** ± 0.01ns | 2.65ns &mdash; 2.67ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_packed/musli_zerocopy/report/) |
| `dec/packed/zerocopy` | **6.67ns** ± 0.02ns | 6.64ns &mdash; 6.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/dec_packed/zerocopy/report/) |

<table>
<tr>
<th colspan="3">
<code>zerocopy-zerocopy/enc/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_zerocopy-zerocopy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_zerocopy-zerocopy.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/packed/musli_zerocopy` | **17.86ns** ± 0.02ns | 17.83ns &mdash; 17.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_packed/musli_zerocopy/report/) |
| `enc/packed/zerocopy` | **7.33ns** ± 0.01ns | 7.30ns &mdash; 7.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/enc_packed/zerocopy/report/) |



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
| `dec/primitives/derive_bitcode` | **242.59ns** ± 0.26ns | 242.17ns &mdash; 243.16ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/derive_bitcode/report/) |
| `dec/primitives/musli_descriptive` | **1.21μs** ± 2.18ns | 1.20μs &mdash; 1.21μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **22.21ns** ± 0.02ns | 22.17ns &mdash; 22.25ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **865.59ns** ± 1.34ns | 863.12ns &mdash; 868.33ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_wire` | **869.22ns** ± 1.19ns | 867.27ns &mdash; 871.86ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/derive_bitcode` | **1.29μs** ± 2.15ns | 1.29μs &mdash; 1.30μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/derive_bitcode/report/) |
| `enc/primitives/musli_descriptive` | **973.08ns** ± 1.70ns | 970.45ns &mdash; 976.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **20.05ns** ± 0.02ns | 20.01ns &mdash; 20.09ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **571.74ns** ± 1.17ns | 569.78ns &mdash; 574.30ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_wire` | **840.35ns** ± 1.13ns | 838.31ns &mdash; 842.70ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_primitives/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bitcode-derive/dec/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/packed/derive_bitcode` | **243.67ns** ± 0.32ns | 243.15ns &mdash; 244.38ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_packed/derive_bitcode/report/) |
| `dec/packed/musli_descriptive` | **1.18μs** ± 1.30ns | 1.18μs &mdash; 1.19μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_packed/musli_descriptive/report/) |
| `dec/packed/musli_packed` | **26.10ns** ± 0.02ns | 26.06ns &mdash; 26.13ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_packed/musli_packed/report/) |
| `dec/packed/musli_storage` | **893.12ns** ± 1.57ns | 890.48ns &mdash; 896.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_packed/musli_storage/report/) |
| `dec/packed/musli_wire` | **888.71ns** ± 1.33ns | 886.61ns &mdash; 891.72ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_packed/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bitcode-derive/enc/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/packed/derive_bitcode` | **1.31μs** ± 1.91ns | 1.31μs &mdash; 1.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_packed/derive_bitcode/report/) |
| `enc/packed/musli_descriptive` | **925.67ns** ± 1.55ns | 923.32ns &mdash; 929.22ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_packed/musli_descriptive/report/) |
| `enc/packed/musli_packed` | **21.65ns** ± 0.02ns | 21.62ns &mdash; 21.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_packed/musli_packed/report/) |
| `enc/packed/musli_storage` | **542.05ns** ± 0.70ns | 540.87ns &mdash; 543.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_packed/musli_storage/report/) |
| `enc/packed/musli_wire` | **581.65ns** ± 0.82ns | 580.31ns &mdash; 583.47ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_packed/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bitcode-derive/dec/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/full_enum/derive_bitcode` | **3.09μs** ± 4.02ns | 3.09μs &mdash; 3.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_full_enum/derive_bitcode/report/) |
| `dec/full_enum/musli_descriptive` | **2.56μs** ± 4.40ns | 2.55μs &mdash; 2.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_full_enum/musli_descriptive/report/) |
| `dec/full_enum/musli_packed` | **582.73ns** ± 1.07ns | 580.79ns &mdash; 584.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_full_enum/musli_packed/report/) |
| `dec/full_enum/musli_storage` | **1.85μs** ± 2.43ns | 1.84μs &mdash; 1.85μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_full_enum/musli_storage/report/) |
| `dec/full_enum/musli_wire` | **1.94μs** ± 2.72ns | 1.93μs &mdash; 1.94μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_full_enum/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bitcode-derive/enc/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_bitcode-derive.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/full_enum/derive_bitcode` | **13.36μs** ± 21.70ns | 13.32μs &mdash; 13.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_full_enum/derive_bitcode/report/) |
| `enc/full_enum/musli_descriptive` | **1.41μs** ± 2.07ns | 1.41μs &mdash; 1.42μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_full_enum/musli_descriptive/report/) |
| `enc/full_enum/musli_packed` | **144.82ns** ± 0.12ns | 144.58ns &mdash; 145.07ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_full_enum/musli_packed/report/) |
| `enc/full_enum/musli_storage` | **924.67ns** ± 1.31ns | 922.40ns &mdash; 927.53ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_full_enum/musli_storage/report/) |
| `enc/full_enum/musli_wire` | **1.40μs** ± 1.64ns | 1.40μs &mdash; 1.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_full_enum/musli_wire/report/) |


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
| `dec/large/derive_bitcode` | **36.77μs** ± 79.92ns | 36.65μs &mdash; 36.95μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/derive_bitcode/report/) |
| `dec/large/musli_descriptive` | **195.23μs** ± 294.64ns | 194.80μs &mdash; 195.91μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **41.62μs** ± 48.03ns | 41.54μs &mdash; 41.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **150.64μs** ± 216.53ns | 150.28μs &mdash; 151.12μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_storage/report/) |
| `dec/large/musli_wire` | **177.35μs** ± 240.67ns | 176.95μs &mdash; 177.89μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_large/musli_wire/report/) |

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
| `enc/large/derive_bitcode` | **83.83μs** ± 234.76ns | 83.44μs &mdash; 84.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/derive_bitcode/report/) |
| `enc/large/musli_descriptive` | **103.40μs** ± 137.45ns | 103.16μs &mdash; 103.69μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **10.43μs** ± 10.46ns | 10.41μs &mdash; 10.45μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **68.44μs** ± 95.17ns | 68.29μs &mdash; 68.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_storage/report/) |
| `enc/large/musli_wire` | **105.11μs** ± 143.61ns | 104.89μs &mdash; 105.44μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_large/musli_wire/report/) |


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
| `dec/allocated/derive_bitcode` | **3.96μs** ± 5.30ns | 3.95μs &mdash; 3.98μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/derive_bitcode/report/) |
| `dec/allocated/musli_descriptive` | **3.83μs** ± 8.02ns | 3.82μs &mdash; 3.85μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **2.69μs** ± 2.50ns | 2.68μs &mdash; 2.69μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **3.52μs** ± 5.74ns | 3.51μs &mdash; 3.53μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_wire` | **3.74μs** ± 4.75ns | 3.73μs &mdash; 3.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_allocated/musli_wire/report/) |

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
| `enc/allocated/derive_bitcode` | **7.22μs** ± 11.13ns | 7.21μs &mdash; 7.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/derive_bitcode/report/) |
| `enc/allocated/musli_descriptive` | **588.06ns** ± 0.68ns | 586.89ns &mdash; 589.55ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **171.90ns** ± 0.12ns | 171.68ns &mdash; 172.15ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **522.95ns** ± 1.14ns | 521.12ns &mdash; 525.50ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_wire` | **557.36ns** ± 0.99ns | 555.78ns &mdash; 559.60ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_allocated/musli_wire/report/) |


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
| `dec/mesh/derive_bitcode` | **345.66ns** ± 0.40ns | 345.00ns &mdash; 346.56ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/derive_bitcode/report/) |
| `dec/mesh/musli_descriptive` | **8.11μs** ± 11.79ns | 8.09μs &mdash; 8.14μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **87.20ns** ± 0.08ns | 87.05ns &mdash; 87.35ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **4.97μs** ± 5.21ns | 4.96μs &mdash; 4.98μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_wire` | **5.72μs** ± 14.10ns | 5.70μs &mdash; 5.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/dec_mesh/musli_wire/report/) |

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
| `enc/mesh/derive_bitcode` | **1.79μs** ± 2.03ns | 1.79μs &mdash; 1.80μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/derive_bitcode/report/) |
| `enc/mesh/musli_descriptive` | **3.39μs** ± 5.51ns | 3.38μs &mdash; 3.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **33.66ns** ± 0.02ns | 33.62ns &mdash; 33.71ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **2.54μs** ± 4.52ns | 2.53μs &mdash; 2.55μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_wire` | **4.72μs** ± 4.42ns | 4.71μs &mdash; 4.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/enc_mesh/musli_wire/report/) |



### BSON

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `u64` - Format is limited to the bounds of signed 64-bit integers.
> - `empty` - Empty variants are not supported.
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
| `dec/primitives/bson`[^bson] | **2.81μs** ± 5.28ns | 2.80μs &mdash; 2.82μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/bson/report/) |
| `dec/primitives/musli_descriptive` | **894.29ns** ± 1.14ns | 892.38ns &mdash; 896.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_descriptive/report/) |
| `dec/primitives/musli_packed` | **15.34ns** ± 0.01ns | 15.32ns &mdash; 15.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_packed/report/) |
| `dec/primitives/musli_storage` | **603.48ns** ± 1.16ns | 601.75ns &mdash; 606.11ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_storage/report/) |
| `dec/primitives/musli_wire` | **613.57ns** ± 1.19ns | 611.57ns &mdash; 616.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_primitives/musli_wire/report/) |

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
| `enc/primitives/bson`[^bson] | **1.35μs** ± 1.63ns | 1.34μs &mdash; 1.35μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/bson/report/) |
| `enc/primitives/musli_descriptive` | **329.47ns** ± 0.45ns | 328.68ns &mdash; 330.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_descriptive/report/) |
| `enc/primitives/musli_packed` | **17.88ns** ± 0.02ns | 17.85ns &mdash; 17.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_packed/report/) |
| `enc/primitives/musli_storage` | **254.92ns** ± 0.31ns | 254.39ns &mdash; 255.59ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_storage/report/) |
| `enc/primitives/musli_wire` | **494.41ns** ± 0.99ns | 492.55ns &mdash; 496.45ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_primitives/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bson/dec/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/packed/bson`[^bson] | **4.03μs** ± 5.64ns | 4.02μs &mdash; 4.04μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_packed/bson/report/) |
| `dec/packed/musli_descriptive` | **923.24ns** ± 1.18ns | 921.19ns &mdash; 925.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_packed/musli_descriptive/report/) |
| `dec/packed/musli_packed` | **22.30ns** ± 0.02ns | 22.27ns &mdash; 22.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_packed/musli_packed/report/) |
| `dec/packed/musli_storage` | **599.07ns** ± 0.71ns | 597.87ns &mdash; 600.63ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_packed/musli_storage/report/) |
| `dec/packed/musli_wire` | **587.26ns** ± 0.63ns | 586.15ns &mdash; 588.62ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_packed/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bson/enc/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/packed/bson`[^bson] | **2.40μs** ± 3.74ns | 2.39μs &mdash; 2.41μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_packed/bson/report/) |
| `enc/packed/musli_descriptive` | **337.91ns** ± 0.38ns | 337.27ns &mdash; 338.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_packed/musli_descriptive/report/) |
| `enc/packed/musli_packed` | **19.54ns** ± 0.02ns | 19.50ns &mdash; 19.58ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_packed/musli_packed/report/) |
| `enc/packed/musli_storage` | **250.07ns** ± 0.32ns | 249.53ns &mdash; 250.76ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_packed/musli_storage/report/) |
| `enc/packed/musli_wire` | **284.14ns** ± 0.38ns | 283.49ns &mdash; 284.98ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_packed/musli_wire/report/) |


<table>
<tr>
<th colspan="3">
<code>bson/dec/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/full_enum/bson`[^bson] | **9.78μs** ± 12.25ns | 9.76μs &mdash; 9.81μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_full_enum/bson/report/) |
| `dec/full_enum/musli_descriptive` | **2.41μs** ± 2.31ns | 2.41μs &mdash; 2.42μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_full_enum/musli_descriptive/report/) |
| `dec/full_enum/musli_packed` | **743.34ns** ± 1.13ns | 741.34ns &mdash; 745.73ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_full_enum/musli_packed/report/) |
| `dec/full_enum/musli_storage` | **1.71μs** ± 2.42ns | 1.71μs &mdash; 1.72μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_full_enum/musli_storage/report/) |
| `dec/full_enum/musli_wire` | **1.88μs** ± 2.76ns | 1.87μs &mdash; 1.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_full_enum/musli_wire/report/) |

<table>
<tr>
<th colspan="3">
<code>bson/enc/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_bson.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_bson.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/full_enum/bson`[^bson] | **6.63μs** ± 8.58ns | 6.62μs &mdash; 6.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_full_enum/bson/report/) |
| `enc/full_enum/musli_descriptive` | **829.47ns** ± 1.06ns | 827.66ns &mdash; 831.80ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_full_enum/musli_descriptive/report/) |
| `enc/full_enum/musli_packed` | **117.25ns** ± 0.15ns | 116.98ns &mdash; 117.57ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_full_enum/musli_packed/report/) |
| `enc/full_enum/musli_storage` | **606.37ns** ± 0.70ns | 605.18ns &mdash; 607.92ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_full_enum/musli_storage/report/) |
| `enc/full_enum/musli_wire` | **1.09μs** ± 1.27ns | 1.09μs &mdash; 1.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_full_enum/musli_wire/report/) |


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
| `dec/large/bson`[^bson] | **2.08ms** ± 1.54μs | 2.08ms &mdash; 2.08ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/bson/report/) |
| `dec/large/musli_descriptive` | **483.76μs** ± 575.58ns | 482.83μs &mdash; 485.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_descriptive/report/) |
| `dec/large/musli_packed` | **105.56μs** ± 92.55ns | 105.39μs &mdash; 105.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_packed/report/) |
| `dec/large/musli_storage` | **347.21μs** ± 634.82ns | 346.23μs &mdash; 348.65μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_storage/report/) |
| `dec/large/musli_wire` | **448.98μs** ± 598.08ns | 448.03μs &mdash; 450.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_large/musli_wire/report/) |

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
| `enc/large/bson`[^bson] | **1.12ms** ± 1.63μs | 1.12ms &mdash; 1.13ms | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/bson/report/) |
| `enc/large/musli_descriptive` | **169.80μs** ± 265.09ns | 169.39μs &mdash; 170.40μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_descriptive/report/) |
| `enc/large/musli_packed` | **32.26μs** ± 28.21ns | 32.21μs &mdash; 32.32μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_packed/report/) |
| `enc/large/musli_storage` | **131.41μs** ± 152.96ns | 131.17μs &mdash; 131.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_storage/report/) |
| `enc/large/musli_wire` | **235.26μs** ± 364.90ns | 234.72μs &mdash; 236.10μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_large/musli_wire/report/) |


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
| `dec/allocated/bson`[^bson] | **7.57μs** ± 9.67ns | 7.55μs &mdash; 7.59μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/bson/report/) |
| `dec/allocated/musli_descriptive` | **3.13μs** ± 5.46ns | 3.12μs &mdash; 3.15μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_descriptive/report/) |
| `dec/allocated/musli_packed` | **2.33μs** ± 1.35ns | 2.32μs &mdash; 2.33μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_packed/report/) |
| `dec/allocated/musli_storage` | **2.84μs** ± 3.60ns | 2.83μs &mdash; 2.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_storage/report/) |
| `dec/allocated/musli_wire` | **2.84μs** ± 3.35ns | 2.83μs &mdash; 2.84μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_allocated/musli_wire/report/) |

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
| `enc/allocated/bson`[^bson] | **2.42μs** ± 2.86ns | 2.42μs &mdash; 2.43μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/bson/report/) |
| `enc/allocated/musli_descriptive` | **403.64ns** ± 0.54ns | 402.72ns &mdash; 404.82ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_descriptive/report/) |
| `enc/allocated/musli_packed` | **155.10ns** ± 0.10ns | 154.92ns &mdash; 155.31ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_packed/report/) |
| `enc/allocated/musli_storage` | **341.22ns** ± 0.57ns | 340.22ns &mdash; 342.44ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_storage/report/) |
| `enc/allocated/musli_wire` | **347.26ns** ± 0.93ns | 346.06ns &mdash; 349.34ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_allocated/musli_wire/report/) |


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
| `dec/mesh/bson`[^bson] | **34.50μs** ± 57.26ns | 34.41μs &mdash; 34.63μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/bson/report/) |
| `dec/mesh/musli_descriptive` | **11.34μs** ± 12.33ns | 11.32μs &mdash; 11.37μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/musli_descriptive/report/) |
| `dec/mesh/musli_packed` | **91.64ns** ± 0.07ns | 91.51ns &mdash; 91.78ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/musli_packed/report/) |
| `dec/mesh/musli_storage` | **6.93μs** ± 5.80ns | 6.92μs &mdash; 6.94μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/musli_storage/report/) |
| `dec/mesh/musli_wire` | **7.96μs** ± 11.81ns | 7.94μs &mdash; 7.99μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/dec_mesh/musli_wire/report/) |

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
| `enc/mesh/bson`[^bson] | **12.68μs** ± 21.50ns | 12.65μs &mdash; 12.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/bson/report/) |
| `enc/mesh/musli_descriptive` | **4.75μs** ± 10.54ns | 4.73μs &mdash; 4.77μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/musli_descriptive/report/) |
| `enc/mesh/musli_packed` | **40.25ns** ± 0.03ns | 40.21ns &mdash; 40.32ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/musli_packed/report/) |
| `enc/mesh/musli_storage` | **3.54μs** ± 9.21ns | 3.53μs &mdash; 3.56μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/musli_storage/report/) |
| `enc/mesh/musli_wire` | **6.63μs** ± 12.47ns | 6.62μs &mdash; 6.66μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-bson/enc_mesh/musli_wire/report/) |



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
| `dec/primitives/miniserde` | **2.08μs** ± 2.86ns | 2.07μs &mdash; 2.08μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/miniserde/report/) |
| `dec/primitives/musli_json` | **2.60μs** ± 4.45ns | 2.59μs &mdash; 2.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/musli_json/report/) |
| `dec/primitives/serde_json` | **2.22μs** ± 4.51ns | 2.21μs &mdash; 2.23μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_primitives/serde_json/report/) |

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
| `enc/primitives/miniserde` | **2.32μs** ± 4.30ns | 2.31μs &mdash; 2.33μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/miniserde/report/) |
| `enc/primitives/musli_json` | **1.00μs** ± 1.07ns | 1.00μs &mdash; 1.00μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/musli_json/report/) |
| `enc/primitives/serde_json` | **960.23ns** ± 0.78ns | 958.84ns &mdash; 961.90ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_primitives/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>miniserde/dec/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_packed_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/packed/miniserde` | **2.79μs** ± 4.09ns | 2.78μs &mdash; 2.79μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_packed/miniserde/report/) |
| `dec/packed/musli_json` | **3.45μs** ± 5.29ns | 3.44μs &mdash; 3.46μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_packed/musli_json/report/) |
| `dec/packed/serde_json` | **2.78μs** ± 4.22ns | 2.78μs &mdash; 2.79μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_packed/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>miniserde/enc/packed</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_packed/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_packed_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/packed/miniserde` | **3.04μs** ± 4.86ns | 3.03μs &mdash; 3.05μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_packed/miniserde/report/) |
| `enc/packed/musli_json` | **913.05ns** ± 1.87ns | 910.07ns &mdash; 917.26ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_packed/musli_json/report/) |
| `enc/packed/serde_json` | **1.13μs** ± 1.84ns | 1.12μs &mdash; 1.13μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_packed/serde_json/report/) |


<table>
<tr>
<th colspan="3">
<code>miniserde/dec/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_full_enum_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `dec/full_enum/miniserde` | **68.30ns** ± 0.09ns | 68.15ns &mdash; 68.49ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_full_enum/miniserde/report/) |
| `dec/full_enum/musli_json` | **60.54ns** ± 0.07ns | 60.43ns &mdash; 60.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_full_enum/musli_json/report/) |
| `dec/full_enum/serde_json` | **71.29ns** ± 0.07ns | 71.18ns &mdash; 71.46ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_full_enum/serde_json/report/) |

<table>
<tr>
<th colspan="3">
<code>miniserde/enc/full_enum</code>
<br />
<a href="https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_full_enum/report/">Report 📓</a>
</th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_miniserde.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_full_enum_miniserde.svg"></a></td>
</tr>
</table>

| Group | Mean | Interval | Link |
|-|-|-|-|
| `enc/full_enum/miniserde` | **97.52ns** ± 0.08ns | 97.39ns &mdash; 97.69ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_full_enum/miniserde/report/) |
| `enc/full_enum/musli_json` | **23.46ns** ± 0.02ns | 23.42ns &mdash; 23.52ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_full_enum/musli_json/report/) |
| `enc/full_enum/serde_json` | **28.14ns** ± 0.03ns | 28.10ns &mdash; 28.21ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_full_enum/serde_json/report/) |


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
| `dec/large/miniserde` | **123.13μs** ± 206.83ns | 122.81μs &mdash; 123.60μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/miniserde/report/) |
| `dec/large/musli_json` | **180.81μs** ± 207.13ns | 180.45μs &mdash; 181.25μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/musli_json/report/) |
| `dec/large/serde_json` | **146.16μs** ± 194.83ns | 145.82μs &mdash; 146.58μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_large/serde_json/report/) |

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
| `enc/large/miniserde` | **102.20μs** ± 133.81ns | 101.97μs &mdash; 102.49μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/miniserde/report/) |
| `enc/large/musli_json` | **63.35μs** ± 115.05ns | 63.16μs &mdash; 63.61μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/musli_json/report/) |
| `enc/large/serde_json` | **62.39μs** ± 61.87ns | 62.28μs &mdash; 62.52μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_large/serde_json/report/) |


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
| `dec/allocated/miniserde` | **672.90ns** ± 0.73ns | 671.58ns &mdash; 674.43ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/miniserde/report/) |
| `dec/allocated/musli_json` | **667.41ns** ± 1.12ns | 665.68ns &mdash; 669.97ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/musli_json/report/) |
| `dec/allocated/serde_json` | **478.08ns** ± 0.58ns | 477.13ns &mdash; 479.37ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_allocated/serde_json/report/) |

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
| `enc/allocated/miniserde` | **742.58ns** ± 1.10ns | 740.63ns &mdash; 744.91ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/miniserde/report/) |
| `enc/allocated/musli_json` | **167.73ns** ± 0.28ns | 167.30ns &mdash; 168.36ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/musli_json/report/) |
| `enc/allocated/serde_json` | **186.23ns** ± 0.24ns | 185.82ns &mdash; 186.74ns | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_allocated/serde_json/report/) |


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
| `dec/mesh/miniserde` | **22.67μs** ± 28.08ns | 22.62μs &mdash; 22.73μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_mesh/miniserde/report/) |
| `dec/mesh/musli_json` | **30.79μs** ± 57.71ns | 30.70μs &mdash; 30.92μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_mesh/musli_json/report/) |
| `dec/mesh/serde_json` | **24.65μs** ± 42.67ns | 24.58μs &mdash; 24.75μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/dec_mesh/serde_json/report/) |

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
| `enc/mesh/miniserde` | **25.78μs** ± 27.94ns | 25.74μs &mdash; 25.85μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_mesh/miniserde/report/) |
| `enc/mesh/musli_json` | **12.84μs** ± 20.78ns | 12.81μs &mdash; 12.88μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_mesh/musli_json/report/) |
| `enc/mesh/serde_json` | **14.04μs** ± 24.13ns | 14.00μs &mdash; 14.09μs | [Report 📓](https://udoprog.github.io/musli/benchmarks/criterion-miniserde/enc_mesh/serde_json/report/) |



## Size comparisons

This is not yet an area which has received much focus, but because people are bound to ask the following section performs a raw size comparison between different formats.
Each test suite serializes a collection of values, which have all been randomly populated.
- A small object containing one of each primitive type and a string and a byte array. (`primitives`)
- Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries. (`packed`)
- An enum with every kind of supported variant. (`full_enum`)
- A really big and complex struct. (`large`)
- A sparse struct which contains fairly plain allocated data like strings and vectors. (`allocated`)
- A mesh containing triangles. (`mesh`)

> **Note** so far these are all synthetic examples. Real world data is
> rarely *this* random. But hopefully it should give an idea of the extreme
> ranges.

#### Full features sizes

These frameworks provide a fair comparison against Müsli on various areas since
they support the same set of features in what types of data they can represent.

| **framework** | `primitives` | `packed` | `large` | `allocated` | `full_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `bincode1` | <a title="samples: 500, min: 93, max: 95, stddev: 0.20591260281973842">94.96 ± 0.21</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 18030, max: 40236, stddev: 7034.590902106533">30208.80 ± 7034.59</a> | <a title="samples: 100, min: 460, max: 1000, stddev: 118.27502652715827">703.59 ± 118.28</a> | <a title="samples: 4000, min: 4, max: 163, stddev: 47.62791866122228">42.81 ± 47.63</a> | <a title="samples: 10, min: 536, max: 920, stddev: 141.09060918431106">699.20 ± 141.09</a> |
| `bincode_derive` | <a title="samples: 500, min: 103, max: 105, stddev: 0.24019991673603594">104.95 ± 0.24</a> | <a title="samples: 500, min: 104, max: 106, stddev: 0.12623787070447653">105.99 ± 0.13</a> | <a title="samples: 10, min: 17881, max: 42139, stddev: 7637.403276637943">31145.70 ± 7637.40</a> | <a title="samples: 100, min: 356, max: 883, stddev: 113.92333343086482">596.71 ± 113.92</a> | <a title="samples: 4000, min: 1, max: 146, stddev: 47.05755374007393">39.06 ± 47.06</a> | - |
| `bincode_serde` | <a title="samples: 500, min: 103, max: 105, stddev: 0.24019991673603594">104.95 ± 0.24</a> | <a title="samples: 500, min: 104, max: 106, stddev: 0.12623787070447653">105.99 ± 0.13</a> | <a title="samples: 10, min: 17881, max: 42139, stddev: 7637.403276637943">31145.70 ± 7637.40</a> | <a title="samples: 100, min: 356, max: 883, stddev: 113.92333343086482">596.71 ± 113.92</a> | <a title="samples: 4000, min: 1, max: 146, stddev: 47.05755374007393">39.06 ± 47.06</a> | <a title="samples: 10, min: 529, max: 913, stddev: 141.09060918431106">692.20 ± 141.09</a> |
| `musli_descriptive` | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 24272, max: 62325, stddev: 11746.630546671671">44494.00 ± 11746.63</a> | <a title="samples: 100, min: 392, max: 936, stddev: 115.89930284518539">641.46 ± 115.90</a> | <a title="samples: 4000, min: 4, max: 191, stddev: 65.1371927530652">54.22 ± 65.14</a> | <a title="samples: 10, min: 1203, max: 2075, stddev: 320.39325835603967">1573.60 ± 320.39</a> |
| `musli_packed` | <a title="samples: 500, min: 95, max: 95, stddev: 0">95.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 25182, max: 59884, stddev: 11282.295849693006">43302.60 ± 11282.30</a> | <a title="samples: 100, min: 461, max: 1001, stddev: 118.27502652715827">704.59 ± 118.28</a> | <a title="samples: 4000, min: 16, max: 191, stddev: 54.28528938856252">59.82 ± 54.29</a> | <a title="samples: 10, min: 536, max: 920, stddev: 141.09060918431106">699.20 ± 141.09</a> |
| `musli_storage` | <a title="samples: 500, min: 122, max: 131, stddev: 1.3556681009745792">127.86 ± 1.36</a> | <a title="samples: 500, min: 127, max: 134, stddev: 1.3827783625729677">130.88 ± 1.38</a> | <a title="samples: 10, min: 20434, max: 50623, stddev: 9394.129614285723">36673.30 ± 9394.13</a> | <a title="samples: 100, min: 369, max: 901, stddev: 114.19128294226314">612.47 ± 114.19</a> | <a title="samples: 4000, min: 2, max: 151, stddev: 53.14677231438912">43.76 ± 53.15</a> | <a title="samples: 10, min: 894, max: 1542, stddev: 238.09040299852492">1169.40 ± 238.09</a> |
| `musli_wire` | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 22877, max: 57794, stddev: 10846.271040777103">41447.90 ± 10846.27</a> | <a title="samples: 100, min: 378, max: 924, stddev: 116.24610918220016">628.39 ± 116.25</a> | <a title="samples: 4000, min: 3, max: 179, stddev: 59.66024592431755">49.77 ± 59.66</a> | <a title="samples: 10, min: 1026, max: 1770, stddev: 273.3630552946027">1342.20 ± 273.36</a> |
| `postcard` | <a title="samples: 500, min: 105, max: 114, stddev: 1.4079360780944647">110.85 ± 1.41</a> | <a title="samples: 500, min: 107, max: 114, stddev: 1.3359101766211645">110.81 ± 1.34</a> | <a title="samples: 10, min: 18204, max: 43171, stddev: 7838.735447124108">31857.70 ± 7838.74</a> | <a title="samples: 100, min: 356, max: 888, stddev: 114.19128294226314">599.47 ± 114.19</a> | <a title="samples: 4000, min: 1, max: 146, stddev: 48.32612079403852">40.01 ± 48.33</a> | <a title="samples: 10, min: 529, max: 913, stddev: 141.09060918431106">692.20 ± 141.09</a> |
| `serde_bitcode` | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 105, max: 105, stddev: 0">105.00 ± 0.00</a> | <a title="samples: 10, min: 16433, max: 37270, stddev: 6598.064050158955">27956.30 ± 6598.06</a> | <a title="samples: 100, min: 357, max: 876, stddev: 113.18496852497684">593.73 ± 113.18</a> | <a title="samples: 4000, min: 1, max: 147, stddev: 47.18087031838193">39.18 ± 47.18</a> | <a title="samples: 10, min: 529, max: 913, stddev: 141.09060918431106">692.20 ± 141.09</a> |
| `serde_rmp` | <a title="samples: 500, min: 111, max: 115, stddev: 0.7291090453423233">113.82 ± 0.73</a> | <a title="samples: 500, min: 116, max: 123, stddev: 1.4824304368165206">119.88 ± 1.48</a> | <a title="samples: 10, min: 20514, max: 48425, stddev: 8759.264095230832">35601.10 ± 8759.26</a> | <a title="samples: 100, min: 362, max: 893, stddev: 114.88570450669658">605.43 ± 114.89</a> | <a title="samples: 4000, min: 6, max: 173, stddev: 51.05828320654722">51.40 ± 51.06</a> | <a title="samples: 10, min: 717, max: 1239, stddev: 191.9722896670246">938.80 ± 191.97</a> |

#### Text-based formats sizes

These are text-based formats, which support the full feature set of this test suite.

| **framework** | `primitives` | `packed` | `large` | `allocated` | `full_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `musli_json`[^incomplete] | <a title="samples: 500, min: 428, max: 442, stddev: 2.370359466410104">435.41 ± 2.37</a> | <a title="samples: 500, min: 443, max: 460, stddev: 2.9921657708088594">452.29 ± 2.99</a> | <a title="samples: 10, min: 46119, max: 135589, stddev: 26685.953122195206">93040.40 ± 26685.95</a> | <a title="samples: 100, min: 624, max: 1294, stddev: 125.8695292753572">927.04 ± 125.87</a> | <a title="samples: 4000, min: 12, max: 508, stddev: 154.95497984172707">110.17 ± 154.95</a> | <a title="samples: 10, min: 2315, max: 3998, stddev: 614.3611722757225">3029.50 ± 614.36</a> |
| `serde_json`[^incomplete] | <a title="samples: 500, min: 428, max: 442, stddev: 2.370359466410104">435.41 ± 2.37</a> | <a title="samples: 500, min: 443, max: 460, stddev: 2.9921657708088594">452.29 ± 2.99</a> | <a title="samples: 10, min: 45838, max: 135320, stddev: 26682.236564613544">92751.90 ± 26682.24</a> | <a title="samples: 100, min: 622, max: 1292, stddev: 125.8695292753572">925.04 ± 125.87</a> | <a title="samples: 4000, min: 7, max: 508, stddev: 155.44015658043512">107.54 ± 155.44</a> | <a title="samples: 10, min: 2315, max: 3998, stddev: 614.3611722757225">3029.50 ± 614.36</a> |

#### Fewer features sizes

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `map` - Maps like `MashMap<K, V>` are not supported.

This is a suite where support for 128-bit integers and maps are disabled.
Usually because the underlying framework lacks support for them.

| **framework** | `primitives` | `packed` | `large` | `allocated` | `full_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `musli_descriptive` | <a title="samples: 500, min: 112, max: 120, stddev: 1.4613363746926964">116.36 ± 1.46</a> | <a title="samples: 500, min: 118, max: 126, stddev: 1.457772273024832">122.33 ± 1.46</a> | <a title="samples: 10, min: 17911, max: 59909, stddev: 12329.34556089657">33591.80 ± 12329.35</a> | <a title="samples: 100, min: 265, max: 730, stddev: 97.87126800036872">494.43 ± 97.87</a> | <a title="samples: 4000, min: 4, max: 182, stddev: 54.906162304426424">48.72 ± 54.91</a> | <a title="samples: 10, min: 1094, max: 1639, stddev: 182.39188578442847">1421.00 ± 182.39</a> |
| `musli_packed` | <a title="samples: 500, min: 63, max: 63, stddev: 0">63.00 ± 0.00</a> | <a title="samples: 500, min: 64, max: 64, stddev: 0">64.00 ± 0.00</a> | <a title="samples: 10, min: 15484, max: 58672, stddev: 13016.95506214875">32171.90 ± 13016.96</a> | <a title="samples: 100, min: 313, max: 783, stddev: 99.62658831858087">548.77 ± 99.63</a> | <a title="samples: 4000, min: 16, max: 190, stddev: 48.587230312474944">55.74 ± 48.59</a> | <a title="samples: 10, min: 488, max: 728, stddev: 80.31936254727125">632.00 ± 80.32</a> |
| `musli_storage` | <a title="samples: 500, min: 84, max: 91, stddev: 1.280818488311287">88.25 ± 1.28</a> | <a title="samples: 500, min: 88, max: 94, stddev: 1.2251938622112004">91.33 ± 1.23</a> | <a title="samples: 10, min: 14230, max: 47327, stddev: 9752.03951796751">26568.20 ± 9752.04</a> | <a title="samples: 100, min: 247, max: 706, stddev: 96.8645363381253">472.96 ± 96.86</a> | <a title="samples: 4000, min: 2, max: 148, stddev: 44.386133192135325">38.78 ± 44.39</a> | <a title="samples: 10, min: 813, max: 1218, stddev: 135.53892429852024">1056.00 ± 135.54</a> |
| `musli_wire` | <a title="samples: 500, min: 96, max: 106, stddev: 1.7524143345681649">101.86 ± 1.75</a> | <a title="samples: 500, min: 102, max: 111, stddev: 1.7655310815729104">106.83 ± 1.77</a> | <a title="samples: 10, min: 16316, max: 55119, stddev: 11435.094954131338">30855.30 ± 11435.09</a> | <a title="samples: 100, min: 253, max: 720, stddev: 98.16000152811735">483.71 ± 98.16</a> | <a title="samples: 4000, min: 3, max: 177, stddev: 50.42654553841163">44.50 ± 50.43</a> | <a title="samples: 10, min: 933, max: 1398, stddev: 155.61876493533805">1212.00 ± 155.62</a> |
| `serde_cbor`[^i128] | <a title="samples: 500, min: 210, max: 213, stddev: 0.5346961754117986">212.69 ± 0.53</a> | <a title="samples: 500, min: 218, max: 222, stddev: 0.847610759724064">221.17 ± 0.85</a> | <a title="samples: 10, min: 24082, max: 66856, stddev: 12008.235377856316">39243.90 ± 12008.24</a> | <a title="samples: 100, min: 344, max: 806, stddev: 97.56187985068759">572.14 ± 97.56</a> | <a title="samples: 4000, min: 6, max: 251, stddev: 80.69812101901833">66.21 ± 80.70</a> | <a title="samples: 10, min: 1060, max: 1587, stddev: 175.81990786028754">1376.60 ± 175.82</a> |

#### Speedy sizes

> **Missing features:**
> - `isize` - `isize` types are not supported.
> - `cstring` - `CString`'s are not supported.

This is a test suite for speedy features.

| **framework** | `primitives` | `packed` | `large` | `allocated` | `full_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `musli_descriptive` | <a title="samples: 500, min: 142, max: 151, stddev: 1.5066187308008552">147.31 ± 1.51</a> | <a title="samples: 500, min: 148, max: 157, stddev: 1.4568459081179361">153.36 ± 1.46</a> | <a title="samples: 10, min: 23377, max: 68953, stddev: 15397.839366612445">46767.80 ± 15397.84</a> | <a title="samples: 100, min: 357, max: 943, stddev: 109.28406059439776">635.71 ± 109.28</a> | <a title="samples: 4000, min: 4, max: 182, stddev: 61.47068097068729">51.89 ± 61.47</a> | <a title="samples: 10, min: 1094, max: 2075, stddev: 348.288687729016">1497.30 ± 348.29</a> |
| `musli_packed` | <a title="samples: 500, min: 87, max: 87, stddev: 0">87.00 ± 0.00</a> | <a title="samples: 500, min: 88, max: 88, stddev: 0">88.00 ± 0.00</a> | <a title="samples: 10, min: 24025, max: 67698, stddev: 15427.674502659174">46003.20 ± 15427.67</a> | <a title="samples: 100, min: 418, max: 1011, stddev: 110.74424364272846">694.45 ± 110.74</a> | <a title="samples: 4000, min: 16, max: 188, stddev: 51.97279987587292">58.07 ± 51.97</a> | <a title="samples: 10, min: 488, max: 920, stddev: 153.37483496323637">665.60 ± 153.37</a> |
| `musli_storage` | <a title="samples: 500, min: 113, max: 120, stddev: 1.3242356285797454">117.32 ± 1.32</a> | <a title="samples: 500, min: 115, max: 123, stddev: 1.2658135723715367">120.35 ± 1.27</a> | <a title="samples: 10, min: 19248, max: 56180, stddev: 12302.367814368094">38383.60 ± 12302.37</a> | <a title="samples: 100, min: 334, max: 914, stddev: 107.59809477867162">607.50 ± 107.60</a> | <a title="samples: 4000, min: 2, max: 146, stddev: 50.107195481287924">41.72 ± 50.11</a> | <a title="samples: 10, min: 813, max: 1542, stddev: 258.8200340004614">1112.70 ± 258.82</a> |
| `musli_wire` | <a title="samples: 500, min: 126, max: 136, stddev: 1.8188908708330995">131.81 ± 1.82</a> | <a title="samples: 500, min: 131, max: 141, stddev: 1.6698970028118476">136.96 ± 1.67</a> | <a title="samples: 10, min: 21869, max: 64077, stddev: 14259.703540045986">43542.50 ± 14259.70</a> | <a title="samples: 100, min: 344, max: 933, stddev: 109.64829000034612">623.65 ± 109.65</a> | <a title="samples: 4000, min: 3, max: 177, stddev: 56.335655474574445">47.57 ± 56.34</a> | <a title="samples: 10, min: 933, max: 1770, stddev: 297.16374274127054">1277.10 ± 297.16</a> |
| `speedy` | <a title="samples: 500, min: 87, max: 87, stddev: 0">87.00 ± 0.00</a> | <a title="samples: 500, min: 88, max: 88, stddev: 0">88.00 ± 0.00</a> | <a title="samples: 10, min: 15393, max: 44466, stddev: 9393.865457840026">30849.60 ± 9393.87</a> | <a title="samples: 100, min: 358, max: 943, stddev: 108.45893739106982">630.17 ± 108.46</a> | <a title="samples: 4000, min: 4, max: 152, stddev: 43.73431921200396">39.57 ± 43.73</a> | <a title="samples: 10, min: 484, max: 916, stddev: 153.37483496323637">661.60 ± 153.37</a> |

#### ε-serde sizes

> **Custom environment:**
> - `MUSLI_VEC_RANGE=10000..20000` - ε-serde benefits from larger inputs, this ensures that the size of the supported suite (primarily `mesh`) reflects that by making the inputs bigger.


This is a test suite for ε-serde features.

| **framework** | `primitives` | `packed` | `large` | `allocated` | `full_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `epserde` | <a title="samples: 500, min: 192, max: 192, stddev: 0">192.00 ± 0.00</a> | <a title="samples: 500, min: 176, max: 176, stddev: 0">176.00 ± 0.00</a> | - | - | - | <a title="samples: 10, min: 568776, max: 899016, stddev: 125339.04410932773">705537.60 ± 125339.04</a> |
| `musli_descriptive` | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 1475531, max: 2672265, stddev: 404103.990138541">2162425.10 ± 404103.99</a> | <a title="samples: 100, min: 357, max: 934, stddev: 119.45318915792913">649.34 ± 119.45</a> | <a title="samples: 4000, min: 4, max: 50035, stddev: 12604.07962230398">4711.84 ± 12604.08</a> | <a title="samples: 10, min: 1291438, max: 2041359, stddev: 284624.50807210896">1602001.10 ± 284624.51</a> |
| `musli_packed` | <a title="samples: 500, min: 95, max: 95, stddev: 0">95.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 1502145, max: 2458826, stddev: 323571.68407875247">2029630.80 ± 323571.68</a> | <a title="samples: 100, min: 407, max: 1000, stddev: 121.77120965154286">712.85 ± 121.77</a> | <a title="samples: 4000, min: 16, max: 20112, stddev: 5059.551836073763">1922.78 ± 5059.55</a> | <a title="samples: 10, min: 568712, max: 898952, stddev: 125339.04410932773">705473.60 ± 125339.04</a> |
| `musli_storage` | <a title="samples: 500, min: 122, max: 131, stddev: 1.3556681009745792">127.86 ± 1.36</a> | <a title="samples: 500, min: 127, max: 134, stddev: 1.3827783625729677">130.88 ± 1.38</a> | <a title="samples: 10, min: 1286408, max: 2136665, stddev: 288816.5798160487">1756605.40 ± 288816.58</a> | <a title="samples: 100, min: 329, max: 902, stddev: 117.92638212037203">620.22 ± 117.93</a> | <a title="samples: 4000, min: 2, max: 20072, stddev: 5050.4238939467705">1906.92 ± 5050.42</a> | <a title="samples: 10, min: 959692, max: 1516973, stddev: 211510.065675017">1190477.50 ± 211510.07</a> |
| `musli_wire` | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 1447312, max: 2589632, stddev: 385125.1673318818">2099262.30 ± 385125.17</a> | <a title="samples: 100, min: 343, max: 923, stddev: 119.8778590899921">636.17 ± 119.88</a> | <a title="samples: 4000, min: 3, max: 45094, stddev: 11363.682493086957">4248.45 ± 11363.68</a> | <a title="samples: 10, min: 1101869, max: 1741710, stddev: 242844.82670234094">1366844.90 ± 242844.83</a> |

#### Müsli vs zerocopy sizes

Compares [`musli-zerocopy`] with [`zerocopy`].

Note that `zerocopy` only supports packed primitives, so we're only comparing with that suite.

| **framework** | `primitives` | `packed` | `large` | `allocated` | `full_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `musli_zerocopy` | <a title="samples: 500, min: 112, max: 112, stddev: 0">112.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - | - |
| `zerocopy` | - | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - | - |

#### Bitcode derive sizes

> **Missing features:**
> - `cstring` - `CString`'s are not supported.

Uses a custom derive-based framework which does not support everything Müsli and serde does.

| **framework** | `primitives` | `packed` | `large` | `allocated` | `full_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `derive_bitcode` | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 106, max: 106, stddev: 0">106.00 ± 0.00</a> | <a title="samples: 10, min: 16431, max: 37268, stddev: 6598.064050158955">27954.30 ± 6598.06</a> | <a title="samples: 100, min: 321, max: 900, stddev: 105.21475704481762">576.07 ± 105.21</a> | <a title="samples: 4000, min: 1, max: 147, stddev: 47.08329850846753">39.10 ± 47.08</a> | <a title="samples: 10, min: 481, max: 913, stddev: 127.80985877466573">620.20 ± 127.81</a> |
| `musli_descriptive` | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 24272, max: 62325, stddev: 11746.630546671671">44494.00 ± 11746.63</a> | <a title="samples: 100, min: 357, max: 943, stddev: 107.18993376245737">621.59 ± 107.19</a> | <a title="samples: 4000, min: 4, max: 191, stddev: 65.06053832729559">54.16 ± 65.06</a> | <a title="samples: 10, min: 1094, max: 2075, stddev: 290.2348876341368">1410.10 ± 290.23</a> |
| `musli_packed` | <a title="samples: 500, min: 95, max: 95, stddev: 0">95.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 25182, max: 59884, stddev: 11282.295849693006">43302.60 ± 11282.30</a> | <a title="samples: 100, min: 418, max: 1011, stddev: 108.84955626919204">679.79 ± 108.85</a> | <a title="samples: 4000, min: 16, max: 191, stddev: 54.16632770400354">59.74 ± 54.17</a> | <a title="samples: 10, min: 488, max: 920, stddev: 127.80985877466573">627.20 ± 127.81</a> |
| `musli_storage` | <a title="samples: 500, min: 122, max: 131, stddev: 1.3556681009745792">127.86 ± 1.36</a> | <a title="samples: 500, min: 127, max: 134, stddev: 1.3827783625729677">130.88 ± 1.38</a> | <a title="samples: 10, min: 20434, max: 50623, stddev: 9394.129614285723">36673.30 ± 9394.13</a> | <a title="samples: 100, min: 334, max: 914, stddev: 105.75013191481136">593.64 ± 105.75</a> | <a title="samples: 4000, min: 2, max: 151, stddev: 53.06677648903792">43.69 ± 53.07</a> | <a title="samples: 10, min: 813, max: 1542, stddev: 215.67913668224844">1047.90 ± 215.68</a> |
| `musli_wire` | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 22877, max: 57794, stddev: 10846.271040777103">41447.90 ± 10846.27</a> | <a title="samples: 100, min: 344, max: 933, stddev: 107.54220752802128">609.56 ± 107.54</a> | <a title="samples: 4000, min: 3, max: 179, stddev: 59.57236437938294">49.70 ± 59.57</a> | <a title="samples: 10, min: 933, max: 1770, stddev: 247.63160137591484">1202.70 ± 247.63</a> |

#### BSON sizes

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `u64` - Format is limited to the bounds of signed 64-bit integers.
> - `empty` - Empty variants are not supported.
> - `number-key` - Maps with numerical keys like `HashMap<u32, T>` are not supported.

Specific comparison to BSON, because the format is limited in capabilities.

| **framework** | `primitives` | `packed` | `large` | `allocated` | `full_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `bson` | <a title="samples: 500, min: 240, max: 241, stddev: 0.22992172581119874">240.94 ± 0.23</a> | <a title="samples: 500, min: 289, max: 289, stddev: 0">289.00 ± 0.00</a> | <a title="samples: 10, min: 65872, max: 116971, stddev: 16042.38189328505">90022.70 ± 16042.38</a> | <a title="samples: 100, min: 521, max: 1078, stddev: 121.78852326882037">759.34 ± 121.79</a> | <a title="samples: 3500, min: 22, max: 305, stddev: 103.08684069506583">100.64 ± 103.09</a> | <a title="samples: 10, min: 1821, max: 3450, stddev: 607.0924558911929">2635.50 ± 607.09</a> |
| `musli_descriptive` | <a title="samples: 500, min: 111, max: 118, stddev: 1.3158054567450292">115.28 ± 1.32</a> | <a title="samples: 500, min: 117, max: 124, stddev: 1.252956503634502">121.39 ± 1.25</a> | <a title="samples: 10, min: 35046, max: 64551, stddev: 8696.71462335059">48964.60 ± 8696.71</a> | <a title="samples: 100, min: 360, max: 898, stddev: 120.13429984812831">590.50 ± 120.13</a> | <a title="samples: 3500, min: 4, max: 183, stddev: 55.11211415110206">54.09 ± 55.11</a> | <a title="samples: 10, min: 1094, max: 2075, stddev: 365.5971143212156">1584.50 ± 365.60</a> |
| `musli_packed` | <a title="samples: 500, min: 63, max: 63, stddev: 0">63.00 ± 0.00</a> | <a title="samples: 500, min: 64, max: 64, stddev: 0">64.00 ± 0.00</a> | <a title="samples: 10, min: 35463, max: 67494, stddev: 9349.629822083865">49661.30 ± 9349.63</a> | <a title="samples: 100, min: 419, max: 974, stddev: 123.35546197878713">658.50 ± 123.36</a> | <a title="samples: 3500, min: 16, max: 191, stddev: 48.83977827090391">60.90 ± 48.84</a> | <a title="samples: 10, min: 488, max: 920, stddev: 160.99689437998487">704.00 ± 161.00</a> |
| `musli_storage` | <a title="samples: 500, min: 84, max: 89, stddev: 1.0394537026726993">87.24 ± 1.04</a> | <a title="samples: 500, min: 87, max: 92, stddev: 0.9957911427603747">90.38 ± 1.00</a> | <a title="samples: 10, min: 28183, max: 52693, stddev: 7061.771168764958">39724.40 ± 7061.77</a> | <a title="samples: 100, min: 342, max: 872, stddev: 118.64781961755557">569.57 ± 118.65</a> | <a title="samples: 3500, min: 2, max: 149, stddev: 44.45512611933154">43.07 ± 44.46</a> | <a title="samples: 10, min: 813, max: 1542, stddev: 271.68225926622443">1177.50 ± 271.68</a> |
| `musli_wire` | <a title="samples: 500, min: 95, max: 104, stddev: 1.6029210835221925">100.74 ± 1.60</a> | <a title="samples: 500, min: 101, max: 109, stddev: 1.4871233977044382">105.91 ± 1.49</a> | <a title="samples: 10, min: 32480, max: 60285, stddev: 8116.21985655884">45522.80 ± 8116.22</a> | <a title="samples: 100, min: 347, max: 887, stddev: 120.4613631003734">579.40 ± 120.46</a> | <a title="samples: 3500, min: 3, max: 177, stddev: 50.50468332375432">49.42 ± 50.50</a> | <a title="samples: 10, min: 933, max: 1770, stddev: 311.9314828612207">1351.50 ± 311.93</a> |

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

| **framework** | `primitives` | `packed` | `large` | `allocated` | `full_enum` | `mesh` |
| - | - | - | - | - | - | - |
| `miniserde` | <a title="samples: 500, min: 312, max: 326, stddev: 2.2674205609017446">319.30 ± 2.27</a> | <a title="samples: 500, min: 347, max: 361, stddev: 2.460792555255309">355.35 ± 2.46</a> | <a title="samples: 10, min: 11381, max: 32089, stddev: 7047.0232304143865">20566.30 ± 7047.02</a> | <a title="samples: 100, min: 42, max: 154, stddev: 32.055788868783125">98.08 ± 32.06</a> | <a title="samples: 500, min: 7, max: 7, stddev: 0">7.00 ± 0.00</a> | <a title="samples: 10, min: 3450, max: 5975, stddev: 859.0277120093391">4874.70 ± 859.03</a> |
| `musli_json`[^incomplete] | <a title="samples: 500, min: 302, max: 317, stddev: 2.3087754329947305">310.67 ± 2.31</a> | <a title="samples: 500, min: 339, max: 353, stddev: 2.5256729796234514">346.68 ± 2.53</a> | <a title="samples: 10, min: 11086, max: 31243, stddev: 6860.941743667556">20023.70 ± 6860.94</a> | <a title="samples: 100, min: 42, max: 154, stddev: 32.055788868783125">98.08 ± 32.06</a> | <a title="samples: 500, min: 7, max: 7, stddev: 0">7.00 ± 0.00</a> | <a title="samples: 10, min: 2294, max: 4011, stddev: 577.4698260515436">3261.00 ± 577.47</a> |
| `serde_json`[^incomplete] | <a title="samples: 500, min: 302, max: 317, stddev: 2.3087754329947305">310.67 ± 2.31</a> | <a title="samples: 500, min: 339, max: 353, stddev: 2.5256729796234514">346.68 ± 2.53</a> | <a title="samples: 10, min: 11086, max: 31243, stddev: 6860.941743667556">20023.70 ± 6860.94</a> | <a title="samples: 100, min: 42, max: 154, stddev: 32.055788868783125">98.08 ± 32.06</a> | <a title="samples: 500, min: 7, max: 7, stddev: 0">7.00 ± 0.00</a> | <a title="samples: 10, min: 2294, max: 4011, stddev: 577.4698260515436">3261.00 ± 577.47</a> |


[^bincode1]: Version 1 of bincode serialization.

[^bincode_serde]: bincode 2 is shifting away from serde, and the serde implementation does not support borrowing from its input.

[^bson]: BSON does not support serializing directly in-place [without patches](https://github.com/mongodb/bson-rust/pull/328). As a result it is expected to be much slower.

[^i128]: Lacks 128-bit support.

[^incomplete]: These formats do not support a wide range of Rust types. Exact level of support varies. But from a size perspective it makes size comparisons either unfair or simply an esoteric exercise since they can (or cannot) make stricter assumptions as a result.

[^musli_value]: `musli-value` is a heap-allocated, in-memory format. Deserialization is expected to be as fast as a dynamic in-memory structure can be traversed, but serialization requires a lot of allocations. It is only included for reference.

[`rkyv`]: https://docs.rs/rkyv
[`zerocopy`]: https://docs.rs/zerocopy
[`musli-zerocopy`]: https://docs.rs/musli-zerocopy
