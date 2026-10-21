mod a {
    #[derive(musli::Encode, musli::Decode)]
    pub struct Body1;

    musli_web_macros::define! {
        #[musli(id = 10)]
        pub type A1;

        impl Broadcast for A1 {
            impl Event for Body1;
        }
    }
}

mod b {
    #[derive(musli::Encode, musli::Decode)]
    pub struct Body2;

    musli_web_macros::define! {
        #[musli(id = 10)]
        pub type B1;

        impl Endpoint for B1 {
            impl Request for Body2;
            type Response<'de> = ();
        }
    }
}

mod c {
    #[derive(musli::Encode, musli::Decode)]
    pub struct Body3;

    musli_web_macros::define! {
        #[musli(id = 20)]
        pub type C1;

        impl Endpoint for C1 {
            impl Request for Body3;
            type Response<'de> = ();
        }
    }
}

// The broadcast `a::A1` and the endpoint `b::B1` share an id.
mod ab {
    musli_web_macros::define! {
        include super::a as A;
        include super::b as B;
    }
}

// `Own` uses the id of `c::C1`.
mod own {
    #[derive(musli::Encode, musli::Decode)]
    pub struct Body4;

    musli_web_macros::define! {
        #[musli(id = 20)]
        pub type Own;

        impl Endpoint for Own {
            impl Request for Body4;
            type Response<'de> = ();
        }

        include super::c as C;
    }
}

fn main() {}
