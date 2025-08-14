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
pub struct TickBody<'de> {
    pub message: &'de str,
    pub tick: u32,
}

api::define! {
    endpoint Hello {
        request<'de> = HelloRequest<'de>;
        response<'de> = HelloResponse<'de>;
    }

    broadcast Tick {
        body<'de> = TickBody<'de>;
    }
}
