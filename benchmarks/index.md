# Benchmarks and size comparisons

> The following are the results of preliminary benchmarking and should be
> taken with a big grain of 🧂.

Identifiers which are used in tests:

- `dec` - Decode a type.
- `enc` - Encode a type.
- `primitives` - A small object containing one of each primitive type and a string and a byte array.
- `primpacked` - Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries.
- `medium_enum` - A moderately sized enum with many field variations.
- `large` - A really big and complex struct.
- `allocated` - A sparse struct which contains fairly plain allocated data like strings and vectors.

The following are one section for each kind of benchmark we perform. They range from "Full features" to more specialized ones like zerocopy comparisons.
- [**Full features**](#full-features) ([Full report](https://udoprog.github.io/musli/benchmarks/criterion-full/report/))
- [**Text-based formats**](#text-based-formats) ([Full report](https://udoprog.github.io/musli/benchmarks/criterion-text/report/))
- [**Fewer features**](#fewer-features) ([Full report](https://udoprog.github.io/musli/benchmarks/criterion-fewer/report/))
- [**Müsli vs rkyv**](#müsli-vs-rkyv) ([Full report](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/report/))
- [**Müsli vs zerocopy**](#müsli-vs-zerocopy) ([Full report](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/report/))
- [**Bitcode derive**](#bitcode-derive) ([Full report](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/report/))

Below you'll also find [size comparisons](#size-comparisons).
### Full features

These frameworks provide a fair comparison against Müsli on various areas since
they support the same set of features in what types of data they can represent.

[Full report](https://udoprog.github.io/musli/benchmarks/criterion-full/report/)

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primitives</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_full.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>1.02μs</b> ± 1.09ns</td>
<td>1.02μs &mdash; 1.02μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>445.78ns</b> ± 0.65ns</td>
<td>444.55ns &mdash; 447.12ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>80.09ns</b> ± 0.18ns</td>
<td>79.77ns &mdash; 80.49ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>399.99ns</b> ± 0.36ns</td>
<td>399.36ns &mdash; 400.77ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>915.30ns</b> ± 1.22ns</td>
<td>913.24ns &mdash; 917.96ns</td>
</tr>
<tr>
<td>
<code>postcard</code>
</td>
<td><b>263.31ns</b> ± 0.30ns</td>
<td>262.82ns &mdash; 263.98ns</td>
</tr>
<tr>
<td>
<code>serde_bincode</code>
</td>
<td><b>86.49ns</b> ± 0.09ns</td>
<td>86.32ns &mdash; 86.67ns</td>
</tr>
<tr>
<td>
<code>serde_bitcode</code>
</td>
<td><b>1.30μs</b> ± 1.76ns</td>
<td>1.30μs &mdash; 1.31μs</td>
</tr>
<tr>
<td>
<code>serde_rmp</code>
</td>
<td><b>341.17ns</b> ± 0.40ns</td>
<td>340.47ns &mdash; 342.02ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primitives</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_full.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>863.95ns</b> ± 0.82ns</td>
<td>862.50ns &mdash; 865.72ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>287.35ns</b> ± 0.42ns</td>
<td>286.67ns &mdash; 288.28ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>129.45ns</b> ± 0.10ns</td>
<td>129.27ns &mdash; 129.67ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>1.22μs</b> ± 1.29ns</td>
<td>1.22μs &mdash; 1.22μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>760.42ns</b> ± 0.79ns</td>
<td>758.97ns &mdash; 762.07ns</td>
</tr>
<tr>
<td>
<code>postcard</code>
</td>
<td><b>431.80ns</b> ± 0.34ns</td>
<td>431.22ns &mdash; 432.52ns</td>
</tr>
<tr>
<td>
<code>serde_bincode</code>
</td>
<td><b>101.03ns</b> ± 0.09ns</td>
<td>100.87ns &mdash; 101.23ns</td>
</tr>
<tr>
<td>
<code>serde_bitcode</code>
</td>
<td><b>3.98μs</b> ± 6.33ns</td>
<td>3.97μs &mdash; 4.00μs</td>
</tr>
<tr>
<td>
<code>serde_rmp</code>
</td>
<td><b>226.08ns</b> ± 0.54ns</td>
<td>225.16ns &mdash; 227.26ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_full.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>1.03μs</b> ± 0.92ns</td>
<td>1.03μs &mdash; 1.03μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>458.45ns</b> ± 0.65ns</td>
<td>457.24ns &mdash; 459.79ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>457.78ns</b> ± 0.47ns</td>
<td>456.93ns &mdash; 458.78ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>470.33ns</b> ± 0.40ns</td>
<td>469.64ns &mdash; 471.19ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>936.10ns</b> ± 0.87ns</td>
<td>934.58ns &mdash; 937.97ns</td>
</tr>
<tr>
<td>
<code>postcard</code>
</td>
<td><b>268.44ns</b> ± 0.34ns</td>
<td>267.83ns &mdash; 269.16ns</td>
</tr>
<tr>
<td>
<code>serde_bincode</code>
</td>
<td><b>69.67ns</b> ± 0.11ns</td>
<td>69.49ns &mdash; 69.92ns</td>
</tr>
<tr>
<td>
<code>serde_bitcode</code>
</td>
<td><b>1.55μs</b> ± 1.91ns</td>
<td>1.55μs &mdash; 1.56μs</td>
</tr>
<tr>
<td>
<code>serde_rmp</code>
</td>
<td><b>440.02ns</b> ± 0.39ns</td>
<td>439.35ns &mdash; 440.86ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_full.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>867.52ns</b> ± 1.31ns</td>
<td>865.09ns &mdash; 870.23ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>232.52ns</b> ± 0.25ns</td>
<td>232.09ns &mdash; 233.05ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>235.62ns</b> ± 0.21ns</td>
<td>235.26ns &mdash; 236.06ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>1.54μs</b> ± 2.09ns</td>
<td>1.53μs &mdash; 1.54μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>746.74ns</b> ± 0.89ns</td>
<td>745.11ns &mdash; 748.58ns</td>
</tr>
<tr>
<td>
<code>postcard</code>
</td>
<td><b>420.23ns</b> ± 0.39ns</td>
<td>419.53ns &mdash; 421.07ns</td>
</tr>
<tr>
<td>
<code>serde_bincode</code>
</td>
<td><b>124.36ns</b> ± 0.14ns</td>
<td>124.12ns &mdash; 124.65ns</td>
</tr>
<tr>
<td>
<code>serde_bitcode</code>
</td>
<td><b>4.78μs</b> ± 5.60ns</td>
<td>4.77μs &mdash; 4.79μs</td>
</tr>
<tr>
<td>
<code>serde_rmp</code>
</td>
<td><b>257.57ns</b> ± 0.20ns</td>
<td>257.21ns &mdash; 258.01ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>medium_enum</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_full.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>97.23ns</b> ± 0.08ns</td>
<td>97.09ns &mdash; 97.40ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>93.99ns</b> ± 0.10ns</td>
<td>93.81ns &mdash; 94.20ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>93.91ns</b> ± 0.11ns</td>
<td>93.73ns &mdash; 94.15ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>81.28ns</b> ± 0.08ns</td>
<td>81.14ns &mdash; 81.45ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>91.22ns</b> ± 0.07ns</td>
<td>91.08ns &mdash; 91.38ns</td>
</tr>
<tr>
<td>
<code>postcard</code>
</td>
<td><b>102.93ns</b> ± 0.11ns</td>
<td>102.73ns &mdash; 103.16ns</td>
</tr>
<tr>
<td>
<code>serde_bincode</code>
</td>
<td><b>96.29ns</b> ± 0.13ns</td>
<td>96.08ns &mdash; 96.57ns</td>
</tr>
<tr>
<td>
<code>serde_bitcode</code>
</td>
<td><b>635.63ns</b> ± 0.55ns</td>
<td>634.65ns &mdash; 636.82ns</td>
</tr>
<tr>
<td>
<code>serde_rmp</code>
</td>
<td><b>154.84ns</b> ± 0.15ns</td>
<td>154.58ns &mdash; 155.15ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>medium_enum</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_full.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>34.23ns</b> ± 0.04ns</td>
<td>34.17ns &mdash; 34.31ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>28.50ns</b> ± 0.02ns</td>
<td>28.46ns &mdash; 28.55ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>28.52ns</b> ± 0.02ns</td>
<td>28.48ns &mdash; 28.57ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>136.03ns</b> ± 0.11ns</td>
<td>135.84ns &mdash; 136.26ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>28.34ns</b> ± 0.03ns</td>
<td>28.29ns &mdash; 28.39ns</td>
</tr>
<tr>
<td>
<code>postcard</code>
</td>
<td><b>48.34ns</b> ± 0.05ns</td>
<td>48.25ns &mdash; 48.45ns</td>
</tr>
<tr>
<td>
<code>serde_bincode</code>
</td>
<td><b>18.23ns</b> ± 0.02ns</td>
<td>18.20ns &mdash; 18.27ns</td>
</tr>
<tr>
<td>
<code>serde_bitcode</code>
</td>
<td><b>614.61ns</b> ± 1.59ns</td>
<td>611.58ns &mdash; 617.86ns</td>
</tr>
<tr>
<td>
<code>serde_rmp</code>
</td>
<td><b>46.55ns</b> ± 0.04ns</td>
<td>46.47ns &mdash; 46.63ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>large</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_full.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>330.22μs</b> ± 263.98ns</td>
<td>329.76μs &mdash; 330.79μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>129.70μs</b> ± 121.65ns</td>
<td>129.48μs &mdash; 129.96μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>78.35μs</b> ± 133.20ns</td>
<td>78.13μs &mdash; 78.64μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>152.25μs</b> ± 338.53ns</td>
<td>151.62μs &mdash; 152.94μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>309.24μs</b> ± 252.26ns</td>
<td>308.78μs &mdash; 309.77μs</td>
</tr>
<tr>
<td>
<code>postcard</code>
</td>
<td><b>131.46μs</b> ± 235.52ns</td>
<td>131.05μs &mdash; 131.97μs</td>
</tr>
<tr>
<td>
<code>serde_bincode</code>
</td>
<td><b>98.11μs</b> ± 80.70ns</td>
<td>97.97μs &mdash; 98.28μs</td>
</tr>
<tr>
<td>
<code>serde_bitcode</code>
</td>
<td><b>132.92μs</b> ± 138.23ns</td>
<td>132.67μs &mdash; 133.21μs</td>
</tr>
<tr>
<td>
<code>serde_rmp</code>
</td>
<td><b>276.93μs</b> ± 346.62ns</td>
<td>276.33μs &mdash; 277.68μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>large</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_full.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>208.32μs</b> ± 161.40ns</td>
<td>208.04μs &mdash; 208.67μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>70.62μs</b> ± 61.76ns</td>
<td>70.51μs &mdash; 70.75μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>35.56μs</b> ± 45.84ns</td>
<td>35.48μs &mdash; 35.66μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>770.34μs</b> ± 1.31μs</td>
<td>768.40μs &mdash; 773.32μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>178.83μs</b> ± 155.23ns</td>
<td>178.56μs &mdash; 179.16μs</td>
</tr>
<tr>
<td>
<code>postcard</code>
</td>
<td><b>134.39μs</b> ± 148.78ns</td>
<td>134.12μs &mdash; 134.70μs</td>
</tr>
<tr>
<td>
<code>serde_bincode</code>
</td>
<td><b>44.79μs</b> ± 70.93ns</td>
<td>44.66μs &mdash; 44.94μs</td>
</tr>
<tr>
<td>
<code>serde_bitcode</code>
</td>
<td><b>137.79μs</b> ± 193.67ns</td>
<td>137.46μs &mdash; 138.21μs</td>
</tr>
<tr>
<td>
<code>serde_rmp</code>
</td>
<td><b>136.91μs</b> ± 172.96ns</td>
<td>136.64μs &mdash; 137.30μs</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>allocated</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_full.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>12.10μs</b> ± 17.66ns</td>
<td>12.07μs &mdash; 12.14μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>7.29μs</b> ± 6.63ns</td>
<td>7.28μs &mdash; 7.30μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>7.25μs</b> ± 6.96ns</td>
<td>7.24μs &mdash; 7.26μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>7.60μs</b> ± 10.65ns</td>
<td>7.59μs &mdash; 7.63μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>12.45μs</b> ± 16.56ns</td>
<td>12.43μs &mdash; 12.49μs</td>
</tr>
<tr>
<td>
<code>postcard</code>
</td>
<td><b>9.63μs</b> ± 9.88ns</td>
<td>9.62μs &mdash; 9.66μs</td>
</tr>
<tr>
<td>
<code>serde_bincode</code>
</td>
<td><b>7.31μs</b> ± 6.70ns</td>
<td>7.30μs &mdash; 7.32μs</td>
</tr>
<tr>
<td>
<code>serde_bitcode</code>
</td>
<td><b>8.73μs</b> ± 9.79ns</td>
<td>8.72μs &mdash; 8.76μs</td>
</tr>
<tr>
<td>
<code>serde_rmp</code>
</td>
<td><b>10.46μs</b> ± 18.01ns</td>
<td>10.43μs &mdash; 10.50μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>allocated</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_full.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_full.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>4.87μs</b> ± 7.39ns</td>
<td>4.86μs &mdash; 4.88μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>1.05μs</b> ± 1.46ns</td>
<td>1.05μs &mdash; 1.06μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>1.03μs</b> ± 0.79ns</td>
<td>1.02μs &mdash; 1.03μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>6.84μs</b> ± 7.34ns</td>
<td>6.82μs &mdash; 6.85μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>4.78μs</b> ± 4.46ns</td>
<td>4.78μs &mdash; 4.79μs</td>
</tr>
<tr>
<td>
<code>postcard</code>
</td>
<td><b>6.84μs</b> ± 7.97ns</td>
<td>6.83μs &mdash; 6.86μs</td>
</tr>
<tr>
<td>
<code>serde_bincode</code>
</td>
<td><b>1.54μs</b> ± 1.39ns</td>
<td>1.54μs &mdash; 1.54μs</td>
</tr>
<tr>
<td>
<code>serde_bitcode</code>
</td>
<td><b>6.54μs</b> ± 7.26ns</td>
<td>6.53μs &mdash; 6.56μs</td>
</tr>
<tr>
<td>
<code>serde_rmp</code>
</td>
<td><b>3.79μs</b> ± 4.55ns</td>
<td>3.78μs &mdash; 3.80μs</td>
</tr>
</table>


### Text-based formats

These are text-based formats, which support the full feature set of this test suite.

[Full report](https://udoprog.github.io/musli/benchmarks/criterion-text/report/)

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primitives</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_text.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_json</code>
</td>
<td><b>3.36μs</b> ± 3.40ns</td>
<td>3.35μs &mdash; 3.36μs</td>
</tr>
<tr>
<td>
<code>serde_json</code>
</td>
<td><b>4.44μs</b> ± 4.90ns</td>
<td>4.43μs &mdash; 4.45μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primitives</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_text.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_json</code>
</td>
<td><b>751.52ns</b> ± 1.08ns</td>
<td>749.71ns &mdash; 753.88ns</td>
</tr>
<tr>
<td>
<code>serde_json</code>
</td>
<td><b>1.35μs</b> ± 1.68ns</td>
<td>1.34μs &mdash; 1.35μs</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_text.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_json</code>
</td>
<td><b>4.00μs</b> ± 3.62ns</td>
<td>3.99μs &mdash; 4.00μs</td>
</tr>
<tr>
<td>
<code>serde_json</code>
</td>
<td><b>4.76μs</b> ± 4.21ns</td>
<td>4.75μs &mdash; 4.77μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_text.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_json</code>
</td>
<td><b>866.96ns</b> ± 0.74ns</td>
<td>865.65ns &mdash; 868.55ns</td>
</tr>
<tr>
<td>
<code>serde_json</code>
</td>
<td><b>1.38μs</b> ± 1.83ns</td>
<td>1.38μs &mdash; 1.39μs</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>medium_enum</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_text.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_json</code>
</td>
<td><b>233.39ns</b> ± 0.30ns</td>
<td>232.86ns &mdash; 234.01ns</td>
</tr>
<tr>
<td>
<code>serde_json</code>
</td>
<td><b>217.17ns</b> ± 0.29ns</td>
<td>216.68ns &mdash; 217.81ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>medium_enum</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_text.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_json</code>
</td>
<td><b>52.91ns</b> ± 0.06ns</td>
<td>52.80ns &mdash; 53.05ns</td>
</tr>
<tr>
<td>
<code>serde_json</code>
</td>
<td><b>78.20ns</b> ± 0.10ns</td>
<td>78.02ns &mdash; 78.41ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>large</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_text.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_json</code>
</td>
<td><b>1.16ms</b> ± 2.05μs</td>
<td>1.16ms &mdash; 1.16ms</td>
</tr>
<tr>
<td>
<code>serde_json</code>
</td>
<td><b>911.58μs</b> ± 935.58ns</td>
<td>909.92μs &mdash; 913.56μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>large</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_text.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_json</code>
</td>
<td><b>277.39μs</b> ± 317.28ns</td>
<td>276.85μs &mdash; 278.08μs</td>
</tr>
<tr>
<td>
<code>serde_json</code>
</td>
<td><b>347.57μs</b> ± 455.02ns</td>
<td>346.83μs &mdash; 348.58μs</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>allocated</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_text.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_json</code>
</td>
<td><b>32.20μs</b> ± 29.37ns</td>
<td>32.15μs &mdash; 32.26μs</td>
</tr>
<tr>
<td>
<code>serde_json</code>
</td>
<td><b>27.10μs</b> ± 26.85ns</td>
<td>27.05μs &mdash; 27.15μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>allocated</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_text.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_text.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_json</code>
</td>
<td><b>6.02μs</b> ± 5.92ns</td>
<td>6.01μs &mdash; 6.04μs</td>
</tr>
<tr>
<td>
<code>serde_json</code>
</td>
<td><b>6.42μs</b> ± 7.25ns</td>
<td>6.41μs &mdash; 6.43μs</td>
</tr>
</table>


### Fewer features

> **Missing features:**
> - `128` - 128-bit integers are not supported.
> - `map` - Maps are not supported.

This is a suite where support for 128-bit integers and maps are disabled.
Usually because the underlying framework lacks support for them.

[Full report](https://udoprog.github.io/musli/benchmarks/criterion-fewer/report/)

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primitives</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_fewer.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>791.69ns</b> ± 1.08ns</td>
<td>789.80ns &mdash; 794.01ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>384.15ns</b> ± 0.46ns</td>
<td>383.25ns &mdash; 385.06ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>77.56ns</b> ± 0.10ns</td>
<td>77.38ns &mdash; 77.76ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>340.95ns</b> ± 0.34ns</td>
<td>340.35ns &mdash; 341.69ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>676.47ns</b> ± 0.71ns</td>
<td>675.20ns &mdash; 677.97ns</td>
</tr>
<tr>
<td>
<code>serde_cbor</code>
</td>
<td><b>1.53μs</b> ± 1.78ns</td>
<td>1.52μs &mdash; 1.53μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primitives</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_fewer.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>550.52ns</b> ± 0.80ns</td>
<td>549.05ns &mdash; 552.18ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>262.87ns</b> ± 0.39ns</td>
<td>262.18ns &mdash; 263.70ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>120.80ns</b> ± 0.11ns</td>
<td>120.60ns &mdash; 121.04ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>1.12μs</b> ± 1.09ns</td>
<td>1.12μs &mdash; 1.13μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>421.29ns</b> ± 0.49ns</td>
<td>420.41ns &mdash; 422.34ns</td>
</tr>
<tr>
<td>
<code>serde_cbor</code>
</td>
<td><b>437.60ns</b> ± 0.40ns</td>
<td>436.92ns &mdash; 438.47ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_fewer.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>822.69ns</b> ± 1.08ns</td>
<td>820.89ns &mdash; 825.08ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>436.83ns</b> ± 0.63ns</td>
<td>435.62ns &mdash; 438.08ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>436.74ns</b> ± 0.62ns</td>
<td>435.56ns &mdash; 437.97ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>402.73ns</b> ± 0.35ns</td>
<td>402.13ns &mdash; 403.47ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>723.71ns</b> ± 0.70ns</td>
<td>722.47ns &mdash; 725.21ns</td>
</tr>
<tr>
<td>
<code>serde_cbor</code>
</td>
<td><b>1.71μs</b> ± 2.42ns</td>
<td>1.70μs &mdash; 1.71μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_fewer.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>545.56ns</b> ± 1.01ns</td>
<td>543.66ns &mdash; 547.63ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>215.15ns</b> ± 0.19ns</td>
<td>214.82ns &mdash; 215.56ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>216.48ns</b> ± 0.25ns</td>
<td>216.02ns &mdash; 217.01ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>1.15μs</b> ± 1.05ns</td>
<td>1.15μs &mdash; 1.15μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>428.34ns</b> ± 0.74ns</td>
<td>427.00ns &mdash; 429.91ns</td>
</tr>
<tr>
<td>
<code>serde_cbor</code>
</td>
<td><b>493.24ns</b> ± 0.59ns</td>
<td>492.24ns &mdash; 494.53ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>medium_enum</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_fewer.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>161.42ns</b> ± 0.22ns</td>
<td>161.05ns &mdash; 161.89ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>147.96ns</b> ± 0.17ns</td>
<td>147.66ns &mdash; 148.31ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>148.25ns</b> ± 0.24ns</td>
<td>147.80ns &mdash; 148.74ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>76.52ns</b> ± 0.07ns</td>
<td>76.38ns &mdash; 76.68ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>153.71ns</b> ± 0.16ns</td>
<td>153.42ns &mdash; 154.06ns</td>
</tr>
<tr>
<td>
<code>serde_cbor</code>
</td>
<td><b>415.68ns</b> ± 0.54ns</td>
<td>414.69ns &mdash; 416.81ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>medium_enum</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_fewer.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>46.86ns</b> ± 0.04ns</td>
<td>46.78ns &mdash; 46.95ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>32.96ns</b> ± 0.03ns</td>
<td>32.91ns &mdash; 33.02ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>32.97ns</b> ± 0.03ns</td>
<td>32.92ns &mdash; 33.04ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>141.09ns</b> ± 0.14ns</td>
<td>140.85ns &mdash; 141.38ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>38.33ns</b> ± 0.06ns</td>
<td>38.23ns &mdash; 38.45ns</td>
</tr>
<tr>
<td>
<code>serde_cbor</code>
</td>
<td><b>56.10ns</b> ± 0.09ns</td>
<td>55.94ns &mdash; 56.30ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>large</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_fewer.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>306.03μs</b> ± 296.01ns</td>
<td>305.51μs &mdash; 306.66μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>98.30μs</b> ± 80.17ns</td>
<td>98.15μs &mdash; 98.46μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>38.78μs</b> ± 82.73ns</td>
<td>38.62μs &mdash; 38.95μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>122.25μs</b> ± 318.71ns</td>
<td>121.64μs &mdash; 122.89μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>274.30μs</b> ± 305.37ns</td>
<td>273.82μs &mdash; 274.99μs</td>
</tr>
<tr>
<td>
<code>serde_cbor</code>
</td>
<td><b>515.57μs</b> ± 824.67ns</td>
<td>514.18μs &mdash; 517.38μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>large</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_fewer.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>207.59μs</b> ± 165.55ns</td>
<td>207.30μs &mdash; 207.94μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>78.72μs</b> ± 89.20ns</td>
<td>78.56μs &mdash; 78.91μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>35.76μs</b> ± 37.35ns</td>
<td>35.70μs &mdash; 35.84μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>727.15μs</b> ± 1.48μs</td>
<td>724.56μs &mdash; 730.35μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>175.20μs</b> ± 189.92ns</td>
<td>174.87μs &mdash; 175.61μs</td>
</tr>
<tr>
<td>
<code>serde_cbor</code>
</td>
<td><b>172.43μs</b> ± 154.90ns</td>
<td>172.17μs &mdash; 172.77μs</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>allocated</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_fewer.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>5.63μs</b> ± 7.43ns</td>
<td>5.61μs &mdash; 5.64μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>2.98μs</b> ± 3.70ns</td>
<td>2.97μs &mdash; 2.99μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>2.87μs</b> ± 2.83ns</td>
<td>2.87μs &mdash; 2.88μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>2.94μs</b> ± 4.92ns</td>
<td>2.93μs &mdash; 2.95μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>5.89μs</b> ± 6.17ns</td>
<td>5.88μs &mdash; 5.90μs</td>
</tr>
<tr>
<td>
<code>serde_cbor</code>
</td>
<td><b>5.56μs</b> ± 5.26ns</td>
<td>5.55μs &mdash; 5.57μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>allocated</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_fewer.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_fewer.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>2.93μs</b> ± 4.86ns</td>
<td>2.92μs &mdash; 2.94μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>482.07ns</b> ± 0.48ns</td>
<td>481.21ns &mdash; 483.08ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>460.95ns</b> ± 0.59ns</td>
<td>459.98ns &mdash; 462.26ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>3.59μs</b> ± 4.94ns</td>
<td>3.58μs &mdash; 3.60μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>2.45μs</b> ± 3.74ns</td>
<td>2.44μs &mdash; 2.46μs</td>
</tr>
<tr>
<td>
<code>serde_cbor</code>
</td>
<td><b>1.63μs</b> ± 1.48ns</td>
<td>1.63μs &mdash; 1.64μs</td>
</tr>
</table>


### Müsli vs rkyv

> **Missing features:**
> - `cstring` - `CString` is not supported.
> - `string-key` - Maps with strings as keys like `HashMap<String, T>` are not supported.
> - `string-set` - String sets like `HashSet<String>` are not supported.
> - `tuple` - Tuples like `(u32, u32)` are not supported.
> - `usize` - `usize` and `isize` types are not supported.

Comparison between [`musli-zerocopy`] and [`rkyv`].

Note that `musli-zerocopy` only supports the `primitives` benchmark.

[Full report](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-rkyv/report/)

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primitives</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_zerocopy-rkyv.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_zerocopy-rkyv.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_zerocopy</code>
</td>
<td><b>0.49ns</b> ± 0.00ns</td>
<td>0.49ns &mdash; 0.49ns</td>
</tr>
<tr>
<td>
<code>rkyv</code>
</td>
<td><b>6.48ns</b> ± 0.01ns</td>
<td>6.47ns &mdash; 6.49ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primitives</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_zerocopy-rkyv.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_zerocopy-rkyv.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_zerocopy</code>
</td>
<td><b>17.60ns</b> ± 0.02ns</td>
<td>17.57ns &mdash; 17.65ns</td>
</tr>
<tr>
<td>
<code>rkyv</code>
</td>
<td><b>13.65ns</b> ± 0.02ns</td>
<td>13.61ns &mdash; 13.70ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_zerocopy-rkyv.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_zerocopy-rkyv.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_zerocopy</code>
</td>
<td><b>0.49ns</b> ± 0.00ns</td>
<td>0.49ns &mdash; 0.50ns</td>
</tr>
<tr>
<td>
<code>rkyv</code>
</td>
<td><b>3.95ns</b> ± 0.01ns</td>
<td>3.94ns &mdash; 3.96ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_zerocopy-rkyv.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_zerocopy-rkyv.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_zerocopy</code>
</td>
<td><b>14.95ns</b> ± 0.02ns</td>
<td>14.92ns &mdash; 14.98ns</td>
</tr>
<tr>
<td>
<code>rkyv</code>
</td>
<td><b>12.30ns</b> ± 0.02ns</td>
<td>12.27ns &mdash; 12.34ns</td>
</tr>
</table>


### Müsli vs zerocopy

Compares [`musli-zerocopy`] with [`zerocopy`].

Note that `zerocopy` only supports packed primitives, so we're only comparing with that suite.

[Full report](https://udoprog.github.io/musli/benchmarks/criterion-zerocopy-zerocopy/report/)

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_zerocopy-zerocopy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_zerocopy-zerocopy.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_zerocopy</code>
</td>
<td><b>0.49ns</b> ± 0.00ns</td>
<td>0.49ns &mdash; 0.49ns</td>
</tr>
<tr>
<td>
<code>zerocopy</code>
</td>
<td><b>14.05ns</b> ± 0.01ns</td>
<td>14.03ns &mdash; 14.08ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_zerocopy-zerocopy.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_zerocopy-zerocopy.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>musli_zerocopy</code>
</td>
<td><b>15.39ns</b> ± 0.02ns</td>
<td>15.36ns &mdash; 15.43ns</td>
</tr>
<tr>
<td>
<code>zerocopy</code>
</td>
<td><b>6.16ns</b> ± 0.01ns</td>
<td>6.15ns &mdash; 6.18ns</td>
</tr>
</table>


### Bitcode derive

> **Missing features:**
> - `cstring` - `CString` is not supported.

Uses a custom derive-based framework which does not support everything Müsli and serde does.

[Full report](https://udoprog.github.io/musli/benchmarks/criterion-bitcode-derive/report/)

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primitives</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primitives_bitcode-derive.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>derive_bitcode</code>
</td>
<td><b>250.08ns</b> ± 0.26ns</td>
<td>249.61ns &mdash; 250.64ns</td>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>1.02μs</b> ± 0.99ns</td>
<td>1.02μs &mdash; 1.02μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>446.67ns</b> ± 0.77ns</td>
<td>445.21ns &mdash; 448.24ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>78.85ns</b> ± 0.13ns</td>
<td>78.65ns &mdash; 79.14ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>405.32ns</b> ± 0.46ns</td>
<td>404.49ns &mdash; 406.29ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>919.70ns</b> ± 0.72ns</td>
<td>918.43ns &mdash; 921.24ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primitives</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primitives_bitcode-derive.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>derive_bitcode</code>
</td>
<td><b>1.31μs</b> ± 1.29ns</td>
<td>1.30μs &mdash; 1.31μs</td>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>985.03ns</b> ± 1.30ns</td>
<td>982.69ns &mdash; 987.73ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>288.85ns</b> ± 0.34ns</td>
<td>288.26ns &mdash; 289.60ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>129.35ns</b> ± 0.17ns</td>
<td>129.07ns &mdash; 129.72ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>1.14μs</b> ± 1.46ns</td>
<td>1.14μs &mdash; 1.14μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>763.26ns</b> ± 1.31ns</td>
<td>760.83ns &mdash; 765.96ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_primpacked_bitcode-derive.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>derive_bitcode</code>
</td>
<td><b>256.63ns</b> ± 0.33ns</td>
<td>256.11ns &mdash; 257.37ns</td>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>1.03μs</b> ± 1.04ns</td>
<td>1.03μs &mdash; 1.04μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>458.24ns</b> ± 0.69ns</td>
<td>456.96ns &mdash; 459.65ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>458.18ns</b> ± 0.50ns</td>
<td>457.25ns &mdash; 459.21ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>469.51ns</b> ± 0.41ns</td>
<td>468.79ns &mdash; 470.40ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>938.20ns</b> ± 0.83ns</td>
<td>936.72ns &mdash; 939.97ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>primpacked</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_primpacked_bitcode-derive.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>derive_bitcode</code>
</td>
<td><b>1.37μs</b> ± 1.33ns</td>
<td>1.36μs &mdash; 1.37μs</td>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>980.09ns</b> ± 1.13ns</td>
<td>978.04ns &mdash; 982.46ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>264.23ns</b> ± 0.24ns</td>
<td>263.81ns &mdash; 264.75ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>264.24ns</b> ± 0.24ns</td>
<td>263.82ns &mdash; 264.76ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>1.45μs</b> ± 2.28ns</td>
<td>1.44μs &mdash; 1.45μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>757.28ns</b> ± 0.93ns</td>
<td>755.61ns &mdash; 759.25ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>medium_enum</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_medium_enum_bitcode-derive.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>derive_bitcode</code>
</td>
<td><b>267.65ns</b> ± 0.25ns</td>
<td>267.21ns &mdash; 268.18ns</td>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>43.45ns</b> ± 0.04ns</td>
<td>43.39ns &mdash; 43.53ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>46.53ns</b> ± 0.05ns</td>
<td>46.45ns &mdash; 46.65ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>46.59ns</b> ± 0.05ns</td>
<td>46.49ns &mdash; 46.70ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>40.40ns</b> ± 0.05ns</td>
<td>40.32ns &mdash; 40.51ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>35.16ns</b> ± 0.05ns</td>
<td>35.09ns &mdash; 35.26ns</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>medium_enum</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_medium_enum_bitcode-derive.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>derive_bitcode</code>
</td>
<td><b>1.11μs</b> ± 1.22ns</td>
<td>1.11μs &mdash; 1.11μs</td>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>28.53ns</b> ± 0.03ns</td>
<td>28.48ns &mdash; 28.60ns</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>22.45ns</b> ± 0.03ns</td>
<td>22.40ns &mdash; 22.52ns</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>22.43ns</b> ± 0.03ns</td>
<td>22.38ns &mdash; 22.48ns</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>112.10ns</b> ± 0.15ns</td>
<td>111.83ns &mdash; 112.42ns</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>19.98ns</b> ± 0.02ns</td>
<td>19.94ns &mdash; 20.02ns</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>large</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_large_bitcode-derive.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>derive_bitcode</code>
</td>
<td><b>68.18μs</b> ± 93.60ns</td>
<td>68.02μs &mdash; 68.38μs</td>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>337.94μs</b> ± 369.36ns</td>
<td>337.30μs &mdash; 338.74μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>137.33μs</b> ± 151.40ns</td>
<td>137.09μs &mdash; 137.67μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>86.78μs</b> ± 81.80ns</td>
<td>86.64μs &mdash; 86.96μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>152.16μs</b> ± 310.39ns</td>
<td>151.59μs &mdash; 152.80μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>316.97μs</b> ± 330.79ns</td>
<td>316.42μs &mdash; 317.70μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>large</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_large_bitcode-derive.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>derive_bitcode</code>
</td>
<td><b>132.46μs</b> ± 171.73ns</td>
<td>132.18μs &mdash; 132.84μs</td>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>208.08μs</b> ± 155.55ns</td>
<td>207.81μs &mdash; 208.42μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>77.10μs</b> ± 82.13ns</td>
<td>76.96μs &mdash; 77.28μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>36.29μs</b> ± 43.17ns</td>
<td>36.22μs &mdash; 36.38μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>772.82μs</b> ± 1.27μs</td>
<td>770.92μs &mdash; 775.72μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>179.02μs</b> ± 170.73ns</td>
<td>178.72μs &mdash; 179.38μs</td>
</tr>
</table>

<table>
<tr>
<th colspan="3"><code>dec</code> / <code>allocated</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/dec_allocated_bitcode-derive.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>derive_bitcode</code>
</td>
<td><b>4.56μs</b> ± 5.21ns</td>
<td>4.55μs &mdash; 4.57μs</td>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>11.74μs</b> ± 14.81ns</td>
<td>11.71μs &mdash; 11.77μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>7.16μs</b> ± 9.17ns</td>
<td>7.14μs &mdash; 7.18μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>6.94μs</b> ± 7.15ns</td>
<td>6.93μs &mdash; 6.96μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>7.48μs</b> ± 7.07ns</td>
<td>7.47μs &mdash; 7.50μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>12.17μs</b> ± 12.75ns</td>
<td>12.15μs &mdash; 12.20μs</td>
</tr>
</table>
<table>
<tr>
<th colspan="3"><code>enc</code> / <code>allocated</code></th>
</tr>
<tr>
<td colspan="3">
<a href="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_bitcode-derive.svg"><img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/gh-pages/benchmarks/images/enc_allocated_bitcode-derive.svg"></a></td>
</tr>
<tr>
<th>Group</th>
<th>Mean</th>
<th>Interval</th>
</tr>
<tr>
<td>
<code>derive_bitcode</code>
</td>
<td><b>8.35μs</b> ± 8.10ns</td>
<td>8.34μs &mdash; 8.37μs</td>
</tr>
<tr>
<td>
<code>musli_descriptive</code>
</td>
<td><b>5.35μs</b> ± 7.32ns</td>
<td>5.33μs &mdash; 5.36μs</td>
</tr>
<tr>
<td>
<code>musli_storage</code>
</td>
<td><b>1.05μs</b> ± 1.23ns</td>
<td>1.05μs &mdash; 1.05μs</td>
</tr>
<tr>
<td>
<code>musli_storage_packed</code>
</td>
<td><b>1.02μs</b> ± 1.18ns</td>
<td>1.02μs &mdash; 1.02μs</td>
</tr>
<tr>
<td>
<code>musli_value</code>
</td>
<td><b>6.92μs</b> ± 6.83ns</td>
<td>6.91μs &mdash; 6.94μs</td>
</tr>
<tr>
<td>
<code>musli_wire</code>
</td>
<td><b>4.75μs</b> ± 6.34ns</td>
<td>4.74μs &mdash; 4.77μs</td>
</tr>
</table>


# Size comparisons

This is not yet an area which has received much focus, but because people are bound to ask the following section performs a raw size comparison between different formats.
Each test suite serializes a collection of values, which have all been randomly populated.
- A small object containing one of each primitive type and a string and a byte array. (`primitives`)
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
| musli_descriptive | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 23289, max: 59248, stddev: 9975.321059494776">40283.40 ± 9975.32</a> | <a title="samples: 100, min: 391, max: 884, stddev: 112.04406454605261">650.74 ± 112.04</a> | <a title="samples: 500, min: 4, max: 191, stddev: 68.11469444987618">48.62 ± 68.11</a> |
| musli_storage | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 16558, max: 41000, stddev: 6469.036327769385">28913.30 ± 6469.04</a> | <a title="samples: 100, min: 351, max: 834, stddev: 110.06487677728985">607.27 ± 110.06</a> | <a title="samples: 500, min: 2, max: 133, stddev: 51.14458579361077">37.19 ± 51.14</a> |
| musli_storage_packed | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 15174, max: 37302, stddev: 5836.498762100442">26407.00 ± 5836.50</a> | <a title="samples: 100, min: 339, max: 822, stddev: 110.06487677728985">595.27 ± 110.06</a> | <a title="samples: 500, min: 2, max: 125, stddev: 45.986234636029856">34.33 ± 45.99</a> |
| musli_wire | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 21737, max: 55121, stddev: 9261.080283098727">37478.30 ± 9261.08</a> | <a title="samples: 100, min: 378, max: 871, stddev: 112.15385637596242">637.55 ± 112.15</a> | <a title="samples: 500, min: 3, max: 171, stddev: 61.50687909494355">44.27 ± 61.51</a> |
| postcard | <a title="samples: 500, min: 105, max: 114, stddev: 1.4079360780944647">110.85 ± 1.41</a> | <a title="samples: 500, min: 107, max: 114, stddev: 1.3359101766211645">110.81 ± 1.34</a> | <a title="samples: 10, min: 16466, max: 40653, stddev: 6422.380494022445">28613.30 ± 6422.38</a> | <a title="samples: 100, min: 352, max: 838, stddev: 110.49432383611385">608.62 ± 110.49</a> | <a title="samples: 500, min: 1, max: 130, stddev: 49.549247259670814">36.06 ± 49.55</a> |
| serde_bincode | <a title="samples: 500, min: 93, max: 95, stddev: 0.20591260281973842">94.96 ± 0.21</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 15483, max: 37612, stddev: 5779.989710198453">26929.50 ± 5779.99</a> | <a title="samples: 100, min: 450, max: 954, stddev: 114.3235758712961">713.20 ± 114.32</a> | <a title="samples: 500, min: 4, max: 135, stddev: 45.39418019966879">36.90 ± 45.39</a> |
| serde_bitcode | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 105, max: 105, stddev: 0">105.00 ± 0.00</a> | <a title="samples: 10, min: 14345, max: 34779, stddev: 5332.908344421457">24918.30 ± 5332.91</a> | <a title="samples: 100, min: 347, max: 830, stddev: 110.06487677728985">603.27 ± 110.06</a> | <a title="samples: 500, min: 1, max: 125, stddev: 47.664768162658625">34.87 ± 47.66</a> |
| serde_rmp | <a title="samples: 500, min: 111, max: 115, stddev: 0.7291090453423233">113.82 ± 0.73</a> | <a title="samples: 500, min: 116, max: 123, stddev: 1.4824304368165206">119.88 ± 1.48</a> | <a title="samples: 10, min: 18462, max: 45975, stddev: 7384.956083823383">32070.80 ± 7384.96</a> | <a title="samples: 100, min: 355, max: 844, stddev: 111.63126622949322">614.98 ± 111.63</a> | <a title="samples: 500, min: 8, max: 137, stddev: 49.066420452280774">45.63 ± 49.07</a> |

#### Text-based formats

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_json[^incomplete] | <a title="samples: 500, min: 308, max: 322, stddev: 2.370359466410104">315.41 ± 2.37</a> | <a title="samples: 500, min: 326, max: 343, stddev: 2.9921657708088594">335.29 ± 2.99</a> | <a title="samples: 10, min: 43993, max: 115721, stddev: 21012.90588210017">75833.30 ± 21012.91</a> | <a title="samples: 100, min: 562, max: 1123, stddev: 123.00065690881492">837.72 ± 123.00</a> | <a title="samples: 500, min: 8, max: 373, stddev: 130.11181228466538">83.55 ± 130.11</a> |
| serde_json[^incomplete] | <a title="samples: 500, min: 428, max: 442, stddev: 2.370359466410104">435.41 ± 2.37</a> | <a title="samples: 500, min: 443, max: 460, stddev: 2.9921657708088594">452.29 ± 2.99</a> | <a title="samples: 10, min: 48411, max: 125528, stddev: 21643.36521454092">85431.70 ± 21643.37</a> | <a title="samples: 100, min: 663, max: 1224, stddev: 123.00065690881492">938.72 ± 123.00</a> | <a title="samples: 500, min: 9, max: 506, stddev: 176.48414773004401">111.36 ± 176.48</a> |

#### Fewer features

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_descriptive | <a title="samples: 500, min: 112, max: 120, stddev: 1.4613363746926964">116.36 ± 1.46</a> | <a title="samples: 500, min: 118, max: 126, stddev: 1.457772273024832">122.33 ± 1.46</a> | <a title="samples: 10, min: 14080, max: 56238, stddev: 14058.642853775038">37702.10 ± 14058.64</a> | <a title="samples: 100, min: 316, max: 693, stddev: 85.30907278830313">502.11 ± 85.31</a> | <a title="samples: 500, min: 4, max: 146, stddev: 50.020774684125">34.91 ± 50.02</a> |
| musli_storage | <a title="samples: 500, min: 78, max: 82, stddev: 0.7069257386741584">80.98 ± 0.71</a> | <a title="samples: 500, min: 81, max: 84, stddev: 0.7482539675805259">83.05 ± 0.75</a> | <a title="samples: 10, min: 9722, max: 37021, stddev: 9098.319695965843">24902.10 ± 9098.32</a> | <a title="samples: 100, min: 290, max: 661, stddev: 84.3586978325294">472.01 ± 84.36</a> | <a title="samples: 500, min: 2, max: 125, stddev: 39.58418391226475">27.13 ± 39.58</a> |
| musli_storage_packed | <a title="samples: 500, min: 63, max: 67, stddev: 0.7069257386741584">65.98 ± 0.71</a> | <a title="samples: 500, min: 81, max: 84, stddev: 0.7482539675805259">83.05 ± 0.75</a> | <a title="samples: 10, min: 8532, max: 32726, stddev: 8090.495985413997">22092.10 ± 8090.50</a> | <a title="samples: 100, min: 280, max: 651, stddev: 84.3586978325294">462.01 ± 84.36</a> | <a title="samples: 500, min: 2, max: 125, stddev: 36.488041821944854">25.30 ± 36.49</a> |
| musli_wire | <a title="samples: 500, min: 96, max: 106, stddev: 1.7524143345681649">101.86 ± 1.75</a> | <a title="samples: 500, min: 102, max: 111, stddev: 1.7655310815729104">106.83 ± 1.77</a> | <a title="samples: 10, min: 12824, max: 51587, stddev: 12966.995989819692">34656.00 ± 12967.00</a> | <a title="samples: 100, min: 305, max: 682, stddev: 85.58531649763293">491.44 ± 85.59</a> | <a title="samples: 500, min: 3, max: 128, stddev: 45.57753003399819">31.77 ± 45.58</a> |
| serde_cbor[^i128] | <a title="samples: 500, min: 210, max: 213, stddev: 0.5346961754117986">212.69 ± 0.53</a> | <a title="samples: 500, min: 218, max: 222, stddev: 0.847610759724064">221.17 ± 0.85</a> | <a title="samples: 10, min: 18864, max: 64245, stddev: 14855.324664577345">42668.10 ± 14855.32</a> | <a title="samples: 100, min: 397, max: 771, stddev: 85.06160767349745">580.23 ± 85.06</a> | <a title="samples: 500, min: 8, max: 250, stddev: 78.74014478015638">53.56 ± 78.74</a> |

#### Müsli vs rkyv

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_zerocopy | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 500, min: 80, max: 80, stddev: 0">80.00 ± 0.00</a> | - | - | - |
| rkyv[^incomplete] | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 500, min: 80, max: 80, stddev: 0">80.00 ± 0.00</a> | <a title="samples: 10, min: 8484, max: 22012, stddev: 4192.326781156258">12750.40 ± 4192.33</a> | <a title="samples: 100, min: 312, max: 784, stddev: 86.34712270828716">522.12 ± 86.35</a> | <a title="samples: 500, min: 128, max: 256, stddev: 29.005489135679134">138.24 ± 29.01</a> |

#### Müsli vs zerocopy

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_zerocopy | <a title="samples: 500, min: 112, max: 112, stddev: 0">112.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - |
| zerocopy | - | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - |

#### Bitcode derive

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| derive_bitcode | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 106, max: 106, stddev: 0">106.00 ± 0.00</a> | <a title="samples: 10, min: 14343, max: 34777, stddev: 5332.908344421457">24916.30 ± 5332.91</a> | <a title="samples: 100, min: 368, max: 826, stddev: 111.12212875930699">583.35 ± 111.12</a> | <a title="samples: 500, min: 1, max: 126, stddev: 48.418951826738365">35.65 ± 48.42</a> |
| musli_descriptive | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 23289, max: 59248, stddev: 9975.321059494776">40283.40 ± 9975.32</a> | <a title="samples: 100, min: 406, max: 882, stddev: 113.5311780965916">629.54 ± 113.53</a> | <a title="samples: 500, min: 4, max: 191, stddev: 69.65895560514821">50.15 ± 69.66</a> |
| musli_storage | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 16558, max: 41000, stddev: 6469.036327769385">28913.30 ± 6469.04</a> | <a title="samples: 100, min: 371, max: 829, stddev: 111.12212875930699">586.35 ± 111.12</a> | <a title="samples: 500, min: 2, max: 133, stddev: 52.040440582300974">38.11 ± 52.04</a> |
| musli_storage_packed | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 15174, max: 37302, stddev: 5836.498762100442">26407.00 ± 5836.50</a> | <a title="samples: 100, min: 360, max: 818, stddev: 111.12212875930699">575.35 ± 111.12</a> | <a title="samples: 500, min: 2, max: 126, stddev: 46.655603050437655">35.05 ± 46.66</a> |
| musli_wire | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 21737, max: 55121, stddev: 9261.080283098727">37478.30 ± 9261.08</a> | <a title="samples: 100, min: 395, max: 871, stddev: 113.76857914204608">617.52 ± 113.77</a> | <a title="samples: 500, min: 3, max: 171, stddev: 62.80787883697388">45.57 ± 62.81</a> |


[^i128]: Lacks 128-bit support.
[^incomplete]: These formats do not support a wide range of Rust types. Exact level of support varies. But from a size perspective it makes size comparisons either unfair or simply an esoteric exercise since they can (or cannot) make stricter assumptions as a result.
[`rkyv`]: https://docs.rs/rkyv
[`zerocopy`]: https://docs.rs/zerocopy
[`musli-zerocopy`]: https://docs.rs/musli-zerocopy
