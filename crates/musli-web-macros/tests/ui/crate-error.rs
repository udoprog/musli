mod a {
    musli_web_macros::define! {
        #![musli(crate = renamed)]
        #![musli(crate = other)]
    }
}

mod b {
    musli_web_macros::define! {
        #![musli(path = renamed)]
    }
}

mod c {
    musli_web_macros::define! {
        #![allow(unused)]
    }
}

mod d {
    musli_web_macros::define! {
        pub type Hello;

        #![musli(crate = renamed)]
    }
}

fn main() {}
