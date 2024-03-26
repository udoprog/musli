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

> **Missing features:** `musli`, `serde`

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

> **Missing features:** `musli`, `serde`

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

> **Missing features:** `musli`, `serde`, `model-no-128`, `model-no-map`, `model-no-map-string-key`

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

> **Missing features:** `model-no-cstring`, `model-no-map`, `model-no-map-string-key`, `model-no-tuple`, `model-no-usize`

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

> **Missing features:** `musli`, `model-no-cstring`

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
| musli_descriptive | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 28754, max: 69169, stddev: 13606.13782232122">55043.60 ± 13606.14</a> | <a title="samples: 100, min: 888, max: 4091, stddev: 654.5420826043197">2634.39 ± 654.54</a> | <a title="samples: 500, min: 4, max: 189, stddev: 67.81784131038086">46.02 ± 67.82</a> |
| musli_storage | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 22439, max: 48857, stddev: 8964.162584982492">39176.50 ± 8964.16</a> | <a title="samples: 100, min: 680, max: 2958, stddev: 474.96809366524803">1918.70 ± 474.97</a> | <a title="samples: 500, min: 2, max: 133, stddev: 50.63234456352979">34.82 ± 50.63</a> |
| musli_storage_packed | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 20996, max: 45011, stddev: 8066.011728233477">35923.00 ± 8066.01</a> | <a title="samples: 100, min: 672, max: 2950, stddev: 474.96809366524803">1910.70 ± 474.97</a> | <a title="samples: 500, min: 2, max: 125, stddev: 45.33625480782461">32.00 ± 45.34</a> |
| musli_wire | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 27162, max: 64146, stddev: 12580.785293454459">51422.00 ± 12580.79</a> | <a title="samples: 100, min: 880, max: 4085, stddev: 655.1471623994108">2626.66 ± 655.15</a> | <a title="samples: 500, min: 3, max: 169, stddev: 61.1825666673114">41.74 ± 61.18</a> |
| postcard | <a title="samples: 500, min: 105, max: 114, stddev: 1.4079360780944647">110.85 ± 1.41</a> | <a title="samples: 500, min: 107, max: 114, stddev: 1.3359101766211645">110.81 ± 1.34</a> | <a title="samples: 10, min: 22656, max: 48678, stddev: 8853.61415976549">39100.10 ± 8853.61</a> | <a title="samples: 100, min: 776, max: 3546, stddev: 571.4393065759476">2281.67 ± 571.44</a> | <a title="samples: 500, min: 1, max: 129, stddev: 48.97965010083264">33.63 ± 48.98</a> |
| serde_bincode | <a title="samples: 500, min: 93, max: 95, stddev: 0.20591260281973842">94.96 ± 0.21</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 21908, max: 46008, stddev: 8024.565037433493">36565.60 ± 8024.57</a> | <a title="samples: 100, min: 720, max: 2998, stddev: 474.96809366524803">1958.70 ± 474.97</a> | <a title="samples: 500, min: 4, max: 135, stddev: 44.43533814431936">34.58 ± 44.44</a> |
| serde_bitcode | <a title="samples: 500, min: 103, max: 105, stddev: 0.3823924685450779">104.92 ± 0.38</a> | <a title="samples: 500, min: 105, max: 105, stddev: 0">105.00 ± 0.00</a> | <a title="samples: 10, min: 20429, max: 42403, stddev: 7378.982528777257">33855.20 ± 7378.98</a> | <a title="samples: 100, min: 679, max: 2957, stddev: 474.96928700706525">1917.58 ± 474.97</a> | <a title="samples: 500, min: 1, max: 125, stddev: 47.076341871475094">32.51 ± 47.08</a> |
| serde_rmp | <a title="samples: 500, min: 111, max: 115, stddev: 0.7291090453423233">113.82 ± 0.73</a> | <a title="samples: 500, min: 116, max: 123, stddev: 1.4824304368165206">119.88 ± 1.48</a> | <a title="samples: 10, min: 24338, max: 54314, stddev: 10185.937895451749">43566.70 ± 10185.94</a> | <a title="samples: 100, min: 811, max: 3504, stddev: 559.6411645867375">2282.37 ± 559.64</a> | <a title="samples: 500, min: 8, max: 137, stddev: 48.706500757085855">43.03 ± 48.71</a> |

#### Text-based formats

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_json[^incomplete] | <a title="samples: 500, min: 308, max: 322, stddev: 2.370359466410104">315.41 ± 2.37</a> | <a title="samples: 500, min: 326, max: 343, stddev: 2.9921657708088594">335.29 ± 2.99</a> | <a title="samples: 10, min: 49474, max: 132961, stddev: 27917.641499417536">104410.10 ± 27917.64</a> | <a title="samples: 100, min: 1825, max: 8048, stddev: 1309.343993418078">5204.37 ± 1309.34</a> | <a title="samples: 500, min: 8, max: 374, stddev: 130.1708978074593">80.56 ± 130.17</a> |
| serde_json[^incomplete] | <a title="samples: 500, min: 428, max: 442, stddev: 2.370359466410104">435.41 ± 2.37</a> | <a title="samples: 500, min: 443, max: 460, stddev: 2.9921657708088594">452.29 ± 2.99</a> | <a title="samples: 10, min: 58290, max: 151281, stddev: 29868.570850477598">114993.50 ± 29868.57</a> | <a title="samples: 100, min: 1883, max: 8106, stddev: 1309.3415809482265">5262.38 ± 1309.34</a> | <a title="samples: 500, min: 9, max: 507, stddev: 176.6225341342379">107.79 ± 176.62</a> |
[^incomplete]: These formats do not support a wide range of Rust types. Exact level of support varies. But from a size perspective it makes size comparisons either unfair or simply an esoteric exercise since they can (or cannot) make stricter assumptions as a result.


#### Fewer features

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_descriptive | <a title="samples: 500, min: 112, max: 120, stddev: 1.4613363746926964">116.36 ± 1.46</a> | <a title="samples: 500, min: 118, max: 126, stddev: 1.457772273024832">122.33 ± 1.46</a> | <a title="samples: 10, min: 19585, max: 60295, stddev: 14254.264184446702">35907.60 ± 14254.26</a> | <a title="samples: 100, min: 542, max: 2268, stddev: 460.18009039940006">1419.38 ± 460.18</a> | <a title="samples: 500, min: 4, max: 147, stddev: 53.484924044070546">38.87 ± 53.48</a> |
| musli_storage | <a title="samples: 500, min: 78, max: 82, stddev: 0.7069257386741584">80.98 ± 0.71</a> | <a title="samples: 500, min: 81, max: 84, stddev: 0.7482539675805259">83.05 ± 0.75</a> | <a title="samples: 10, min: 13050, max: 39778, stddev: 9237.644383716013">23640.20 ± 9237.64</a> | <a title="samples: 100, min: 414, max: 1661, stddev: 333.45055105667467">1053.50 ± 333.45</a> | <a title="samples: 500, min: 2, max: 122, stddev: 40.735402342434305">29.39 ± 40.74</a> |
| musli_storage_packed | <a title="samples: 500, min: 63, max: 67, stddev: 0.7069257386741584">65.98 ± 0.71</a> | <a title="samples: 500, min: 81, max: 84, stddev: 0.7482539675805259">83.05 ± 0.75</a> | <a title="samples: 10, min: 11785, max: 35243, stddev: 8224.133459520219">21109.20 ± 8224.13</a> | <a title="samples: 100, min: 408, max: 1655, stddev: 333.45055105667467">1047.50 ± 333.45</a> | <a title="samples: 500, min: 2, max: 122, stddev: 36.834142639676045">26.96 ± 36.83</a> |
| musli_wire | <a title="samples: 500, min: 96, max: 106, stddev: 1.7524143345681649">101.86 ± 1.75</a> | <a title="samples: 500, min: 102, max: 111, stddev: 1.7655310815729104">106.83 ± 1.77</a> | <a title="samples: 10, min: 18110, max: 55370, stddev: 13175.128890830632">33125.10 ± 13175.13</a> | <a title="samples: 100, min: 534, max: 2264, stddev: 460.5688304694533">1413.82 ± 460.57</a> | <a title="samples: 500, min: 3, max: 128, stddev: 48.062208188971134">34.99 ± 48.06</a> |
| serde_cbor[^i128] | <a title="samples: 500, min: 210, max: 213, stddev: 0.5346961754117986">212.69 ± 0.53</a> | <a title="samples: 500, min: 218, max: 222, stddev: 0.847610759724064">221.17 ± 0.85</a> | <a title="samples: 10, min: 21951, max: 68937, stddev: 14937.52257404152">39216.50 ± 14937.52</a> | <a title="samples: 100, min: 560, max: 2065, stddev: 392.00111874840366">1324.73 ± 392.00</a> | <a title="samples: 500, min: 8, max: 250, stddev: 86.93208604422198">61.70 ± 86.93</a> |
[^i128]: Lacks 128-bit support.


#### Müsli vs rkyv

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_zerocopy | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 500, min: 80, max: 80, stddev: 0">80.00 ± 0.00</a> | - | - | - |
| rkyv[^incomplete] | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 500, min: 80, max: 80, stddev: 0">80.00 ± 0.00</a> | <a title="samples: 10, min: 6352, max: 19424, stddev: 3868.9313459920686">13947.20 ± 3868.93</a> | <a title="samples: 100, min: 440, max: 2152, stddev: 432.9606430150437">1286.96 ± 432.96</a> | <a title="samples: 500, min: 128, max: 256, stddev: 30.415154117643404">139.52 ± 30.42</a> |
[^incomplete]: These formats do not support a wide range of Rust types. Exact level of support varies. But from a size perspective it makes size comparisons either unfair or simply an esoteric exercise since they can (or cannot) make stricter assumptions as a result.


#### Müsli vs zerocopy

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_zerocopy | <a title="samples: 500, min: 112, max: 112, stddev: 0">112.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - |
| zerocopy | - | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - |

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
