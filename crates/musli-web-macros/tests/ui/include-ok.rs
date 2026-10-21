use musli_web::api::Id;

mod inner {
    #[derive(musli::Encode, musli::Decode)]
    pub struct Body1;

    musli_web_macros::define! {
        #[musli(id = 30000)]
        pub type Inner;

        impl Endpoint for Inner {
            impl Request for Body1;
            type Response<'de> = ();
        }
    }
}

mod lib {
    #[derive(musli::Encode, musli::Decode)]
    pub struct Body2;
    #[derive(musli::Encode, musli::Decode)]
    pub struct Body3;

    musli_web_macros::define! {
        #[musli(id = 32000)]
        pub type Login;

        impl Endpoint for Login {
            impl Request for Body2;
            type Response<'de> = ();
        }

        #[musli(id = 32100)]
        pub type Session;

        impl Broadcast for Session {
            impl Event for Body3;
        }

        include crate::inner as Inner;
    }
}

mod empty {
    musli_web_macros::define! {}
}

#[derive(musli::Encode, musli::Decode)]
pub struct Body4;

musli_web_macros::define! {
    pub type Hello;

    impl Endpoint for Hello {
        impl Request for Body4;
        type Response<'de> = ();
    }

    /// The library.
    include lib as Lib;
    include self::empty as Empty;
}

fn main() {
    assert_eq!(Request::from_id(Hello::ID), Request::Hello);
    assert_eq!(Request::from_id(lib::Login::ID), Request::Lib(lib::Request::Login));
    assert_eq!(
        Request::from_id(inner::Inner::ID),
        Request::Lib(lib::Request::Inner(inner::Request::Inner))
    );
    assert_eq!(Request::from_id(lib::Session::ID), Request::Unknown(lib::Session::ID));
    assert_eq!(Event::from_id(lib::Session::ID), Event::Lib(lib::Event::Session));
    assert_eq!(Request::Lib(lib::Request::Login).id(), lib::Login::ID);
    assert_eq!(format!("{:?}", debug_id(inner::Inner::ID)), "Lib(Inner(Inner))");
    assert_eq!(format!("{:?}", debug_id(lib::Session::ID)), "Lib(Session)");
    assert!(Request::IDS.contains(inner::Inner::ID));
    assert!(!Request::IDS.contains(lib::Session::ID));
}
