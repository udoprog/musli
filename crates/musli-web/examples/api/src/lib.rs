use musli::{Decode, Encode};
use musli_web::api;

#[derive(Encode, Decode)]
pub struct HelloRequest<'de> {
    pub message: &'de str,
}

#[derive(Encode, Decode)]
pub struct HelloResponse<'de> {
    pub message: &'de str,
}

#[derive(Encode, Decode)]
pub struct TickEvent<'de> {
    pub message: &'de str,
    pub tick: u32,
}

#[derive(Encode)]
pub struct OwnedTickEvent {
    pub message: String,
    pub tick: u32,
}

api::define! {
    endpoint Hello {
        request<'de> = HelloRequest<'de>;
        response<'de> = HelloResponse<'de>;
    }

    broadcast Tick {
        event<'de> = TickEvent<'de>;
        event = OwnedTickEvent;
    }
}
