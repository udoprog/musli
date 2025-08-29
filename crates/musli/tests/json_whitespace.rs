use musli::Decode;

#[derive(Debug, PartialEq, Decode)]
struct Floats {
    f32: f32,
    f64: f64,
}

#[derive(Debug, PartialEq, Decode)]
struct Integers {
    i32: i32,
    i64: i64,
}

#[test]
fn decode_floats() {
    let result: Result<Floats, _> =
        musli::json::from_str("{ \"f32\" : 42.1234578901 , \"f64\" : 42.1234578901 }");
    assert_eq!(
        result,
        Ok(Floats {
            f32: 42.123_46,
            f64: 42.1234578901
        })
    );
}

#[test]
fn decode_integers() {
    let result: Result<Integers, _> = musli::json::from_str("{ \"i32\" : 42 , \"i64\" : 42 }");
    assert_eq!(result, Ok(Integers { i32: 42, i64: 42 }));
}
