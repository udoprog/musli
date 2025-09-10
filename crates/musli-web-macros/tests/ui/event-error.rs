musli_web_macros::define! {
    type Broadcast;

    impl Broadcast for Broadcast {
        impl<'a, 'b> Event for String;
    }

    type Broadcast2;

    impl Broadcast for Broadcast2 {
        impl<T> Event for String;
    }
}

fn main() {}