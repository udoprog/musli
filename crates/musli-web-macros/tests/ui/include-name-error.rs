mod a {
    musli_web_macros::define! {}
}

mod b {
    musli_web_macros::define! {
        pub type Hello;

        include super::a as Request;
        include super::a as Unknown;
        include super::a as Hello;
        include super::a as A;
        include super::a as A;
    }
}

mod c {
    musli_web_macros::define! {
        pub include super::a as A;
    }
}

fn main() {}
