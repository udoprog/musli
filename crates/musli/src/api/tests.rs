use crate::api::Endpoint;

struct Pong;

#[derive(Endpoint)]
#[endpoint(crate, response = Pong)]
struct PingPong;

#[test]
fn test_match() {
    match PingPong::KIND {
        PingPong::KIND => {}
        _ => panic!(),
    }
}
