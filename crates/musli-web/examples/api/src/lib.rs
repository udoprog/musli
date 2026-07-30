use musli::{Decode, Encode};
use musli_web::api::{self, EncodeBody};

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
pub struct OwnedTickEvent<S>
where
    S: AsRef<str>,
{
    pub message: S,
    pub tick: u32,
}

api::define! {
    /// The hello endpoint.
    pub type Hello;

    /// The tick broadcast.
    pub type Tick;

    impl Endpoint for Hello {
        impl<'de> Request for HelloRequest<'de>;
        type Response<'de> = HelloResponse<'de>;
    }

    impl Broadcast for Tick {
        impl<'de> Event for TickEvent<'de>;
        // NB: `EncodeBody` covers every mode a body can be encoded in, which
        // is what allows this event to be sent using any `api::Format`. A
        // generic body has to ask for it explicitly, while a plain
        // `#[derive(Encode)]` type gets it for free.
        impl<S> Event for OwnedTickEvent<S> where S: AsRef<str> + EncodeBody;
    }
}
