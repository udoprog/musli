extern crate musli_web as renamed;

use renamed::api::Id;

mod inner {
    #[derive(musli::Encode, musli::Decode)]
    pub struct Body1;

    musli_web_macros::define! {
        #![musli(crate = ::renamed)]

        #[musli(id = 30000)]
        pub type Inner;

        impl Endpoint for Inner {
            impl Request for Body1;
            type Response<'de> = ();
        }
    }
}

#[derive(musli::Encode, musli::Decode)]
pub struct Body2;

#[derive(musli::Encode, musli::Decode)]
pub struct Body3;

musli_web_macros::define! {
    #![musli(crate = renamed)]

    pub type Hello;

    impl Endpoint for Hello {
        impl Request for Body2;
        type Response<'de> = ();
    }

    pub type Session;

    impl Broadcast for Session {
        impl Event for Body3;
    }

    include inner as Inner;
}

fn main() {
    assert_eq!(Request::from_id(Hello::ID), Request::Hello);
    assert_eq!(Request::from_id(inner::Inner::ID), Request::Inner(inner::Request::Inner));
    assert_eq!(Event::from_id(Session::ID), Event::Session);
    assert_eq!(format!("{:?}", debug_id(inner::Inner::ID)), "Inner(Inner)");
}
