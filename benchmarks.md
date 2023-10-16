Summary of the different kinds of benchmarks we support:

- `primitives` which is a small object containing one of each primitive type and a string and a byte array.
- `primpacked` Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries.
- `medium_enum` A moderately sized enum with many field variations.
- `large` A really big and complex struct.
- `allocated` A sparse struct which contains fairly plain allocated data like strings and vectors.

# Full features

These frameworks provide a fair comparison against Müsli on various areas since
they support the same set of features in what types of data they can represent.

`primitives`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_primitives_full.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_primitives_full.svg">

`primpacked`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_primpacked_full.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_primpacked_full.svg">

`medium_enum`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_medium_enum_full.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_medium_enum_full.svg">

`large`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_large_full.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_large_full.svg">

`allocated`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_allocated_full.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_allocated_full.svg">

# Fewer features

> **Missing features:** `model-no-128`, `model-no-map`

This is a suite where support for 128-bit integers and maps are disabled.
Usually because the underlying framework lacks support for them.

`primitives`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_primitives_fewer.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_primitives_fewer.svg">

`primpacked`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_primpacked_fewer.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_primpacked_fewer.svg">

`medium_enum`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_medium_enum_fewer.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_medium_enum_fewer.svg">

`large`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_large_fewer.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_large_fewer.svg">

`allocated`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_allocated_fewer.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_allocated_fewer.svg">

# Müsli vs rkyv

Comparison between [`musli-zerocopy`] and [`rkyv`].

Note that `musli-zerocopy` only supports the `primitives` benchmark.

`primitives`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_primitives_zerocopy-rkyv.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_primitives_zerocopy-rkyv.svg">

`primpacked`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_primpacked_zerocopy-rkyv.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_primpacked_zerocopy-rkyv.svg">

# Müsli vs zerocopy

Compares [`musli-zerocopy`] with [`zerocopy`].

Note that `zerocopy` only supports packed primitives, so we're only comparing with that suite.

`primpacked`: `dec` - Decode a type, `enc` - Encode a type.

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/dec_primpacked_zerocopy-zerocopy.svg">

<img style="background-color: white;" src="https://raw.githubusercontent.com/udoprog/musli/benchmarks/images/enc_primpacked_zerocopy-zerocopy.svg">

# Size comparisons

This is not yet an area which has received much focus, but because people are bound to ask the following section performs a raw size comparison between different formats.
Each test suite serializes a collection of values, which have all been randomly populated.
- which is a small object containing one of each primitive type and a string and a byte array. (`primitives`)
- Tried to achieve the same goal as `primitives`, but with a packed layout to support certain zerocopy libraries. (`primpacked`)
- A moderately sized enum with many field variations. (`medium_enum`)
- A really big and complex struct. (`large`)
- A sparse struct which contains fairly plain allocated data like strings and vectors. (`allocated`)

#### Full features

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_descriptive | <a title="samples: 500, min: 154, max: 164, stddev: 1.5621574824581534">159.89 ± 1.56</a> | <a title="samples: 500, min: 161, max: 170, stddev: 1.5612764008976794">165.80 ± 1.56</a> | <a title="samples: 10, min: 51160, max: 181513, stddev: 34252.25277978078">111443.90 ± 34252.25</a> | <a title="samples: 100, min: 144, max: 1294, stddev: 319.5851216812197">771.50 ± 319.59</a> | <a title="samples: 500, min: 4, max: 985, stddev: 205.01225144854146">112.77 ± 205.01</a> |
| musli_storage | <a title="samples: 500, min: 113, max: 116, stddev: 0.698558515802362">115.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 42028, max: 158673, stddev: 31799.56734752849">96272.10 ± 31799.57</a> | <a title="samples: 100, min: 108, max: 1217, stddev: 312.23862333157956">687.39 ± 312.24</a> | <a title="samples: 500, min: 2, max: 982, stddev: 202.60038890387142">101.20 ± 202.60</a> |
| musli_storage_packed | <a title="samples: 500, min: 96, max: 99, stddev: 0.698558515802362">98.00 ± 0.70</a> | <a title="samples: 500, min: 116, max: 118, stddev: 0.7276785004382086">117.02 ± 0.73</a> | <a title="samples: 10, min: 39765, max: 154012, stddev: 31392.538540869868">93107.60 ± 31392.54</a> | <a title="samples: 100, min: 104, max: 1213, stddev: 312.23862333157956">683.39 ± 312.24</a> | <a title="samples: 500, min: 2, max: 982, stddev: 202.2586117227151">98.27 ± 202.26</a> |
| musli_wire | <a title="samples: 500, min: 137, max: 147, stddev: 1.7739210805444463">143.30 ± 1.77</a> | <a title="samples: 500, min: 143, max: 153, stddev: 1.8691292090168572">148.43 ± 1.87</a> | <a title="samples: 10, min: 48744, max: 176337, stddev: 33677.705130249">107970.40 ± 33677.71</a> | <a title="samples: 100, min: 133, max: 1273, stddev: 317.37806020580564">755.13 ± 317.38</a> | <a title="samples: 500, min: 3, max: 984, stddev: 203.93184057424665">108.38 ± 203.93</a> |
| postcard | <a title="samples: 500, min: 105, max: 114, stddev: 1.4079360780944647">110.85 ± 1.41</a> | <a title="samples: 500, min: 107, max: 114, stddev: 1.3359101766211645">110.81 ± 1.34</a> | <a title="samples: 10, min: 41851, max: 158559, stddev: 31815.58836733968">96165.80 ± 31815.59</a> | <a title="samples: 100, min: 103, max: 1212, stddev: 312.2503412328">682.38 ± 312.25</a> | <a title="samples: 500, min: 1, max: 982, stddev: 202.60511672709544">99.97 ± 202.61</a> |
| serde_bincode | <a title="samples: 500, min: 93, max: 95, stddev: 0.20591260281973842">94.96 ± 0.21</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | <a title="samples: 10, min: 39830, max: 154902, stddev: 31688.48715054097">93659.10 ± 31688.49</a> | <a title="samples: 100, min: 124, max: 1231, stddev: 311.9260143046745">702.04 ± 311.93</a> | <a title="samples: 500, min: 4, max: 991, stddev: 204.1994578935018">100.59 ± 204.20</a> |
| serde_json[^incomplete] | <a title="samples: 500, min: 428, max: 442, stddev: 2.370359466410104">435.41 ± 2.37</a> | <a title="samples: 500, min: 443, max: 460, stddev: 2.9921657708088594">452.29 ± 2.99</a> | <a title="samples: 10, min: 92095, max: 268121, stddev: 46260.75859310999">169916.30 ± 46260.76</a> | <a title="samples: 100, min: 235, max: 2036, stddev: 418.891792590879">1162.31 ± 418.89</a> | <a title="samples: 500, min: 16, max: 999, stddev: 248.86539754654532">183.15 ± 248.87</a> |
| serde_rmp | <a title="samples: 500, min: 111, max: 115, stddev: 0.7291090453423233">113.82 ± 0.73</a> | <a title="samples: 500, min: 116, max: 123, stddev: 1.4824304368165206">119.88 ± 1.48</a> | <a title="samples: 10, min: 44352, max: 165818, stddev: 32623.697988425534">101004.40 ± 32623.70</a> | <a title="samples: 100, min: 117, max: 1239, stddev: 313.9550802264553">712.74 ± 313.96</a> | <a title="samples: 500, min: 15, max: 997, stddev: 201.98749738535795">116.53 ± 201.99</a> |
[^incomplete]: These formats do not support a wide range of Rust types. Exact level of support varies. But from a size perspective it makes size comparisons either unfair or simply an esoteric exercise since they can (or cannot) make stricter assumptions as a result.


#### Fewer features

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| serde_bitcode[^i128] | <a title="samples: 500, min: 62, max: 63, stddev: 0.21794494717703713">62.95 ± 0.22</a> | <a title="samples: 500, min: 64, max: 64, stddev: 0">64.00 ± 0.00</a> | <a title="samples: 10, min: 11499, max: 25967, stddev: 4601.526903105099">19646.60 ± 4601.53</a> | <a title="samples: 100, min: 147, max: 1229, stddev: 303.8831676812652">723.02 ± 303.88</a> | <a title="samples: 500, min: 1, max: 968, stddev: 201.29608441298603">92.28 ± 201.30</a> |
| serde_cbor[^i128] | <a title="samples: 500, min: 210, max: 213, stddev: 0.5346961754117986">212.69 ± 0.53</a> | <a title="samples: 500, min: 218, max: 222, stddev: 0.847610759724064">221.17 ± 0.85</a> | <a title="samples: 10, min: 24915, max: 51982, stddev: 8302.228534556249">38302.40 ± 8302.23</a> | <a title="samples: 100, min: 203, max: 1329, stddev: 303.70707663799993">799.46 ± 303.71</a> | <a title="samples: 500, min: 15, max: 983, stddev: 207.98828100640665">138.66 ± 207.99</a> |
[^i128]: Lacks 128-bit support.


#### Müsli vs rkyv

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_zerocopy | <a title="samples: 500, min: 88, max: 88, stddev: 0">88.00 ± 0.00</a> | <a title="samples: 500, min: 80, max: 80, stddev: 0">80.00 ± 0.00</a> | - | - | - |
| rkyv[^incomplete] | <a title="samples: 500, min: 88, max: 88, stddev: 0">88.00 ± 0.00</a> | <a title="samples: 500, min: 80, max: 80, stddev: 0">80.00 ± 0.00</a> | <a title="samples: 10, min: 11028, max: 24148, stddev: 4824.707410817779">16744.00 ± 4824.71</a> | <a title="samples: 100, min: 64, max: 1092, stddev: 327.04137719866577">572.76 ± 327.04</a> | <a title="samples: 500, min: 104, max: 1120, stddev: 226.26358632356207">188.75 ± 226.26</a> |
[^incomplete]: These formats do not support a wide range of Rust types. Exact level of support varies. But from a size perspective it makes size comparisons either unfair or simply an esoteric exercise since they can (or cannot) make stricter assumptions as a result.


#### Müsli vs zerocopy

| **framework** | **primitives** | **primpacked** | **large** | **allocated** | **medium_enum** |
| - | - | - | - | - | - |
| musli_zerocopy | <a title="samples: 500, min: 104, max: 104, stddev: 0">104.00 ± 0.00</a> | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - |
| zerocopy | - | <a title="samples: 500, min: 96, max: 96, stddev: 0">96.00 ± 0.00</a> | - | - | - |

[`rkyv`]: https://docs.rs/rkyv
[`zerocopy`]: https://docs.rs/zerocopy
[`musli-zerocopy`]: https://docs.rs/musli-zerocopy
