//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/musli-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/musli)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/musli-web-macros.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/musli-web-macros)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-musli--web--macros-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/musli-web-macros)
//!
//! This crate provides the macros used in [Müsli web].
//!
//! Please refer to <https://docs.rs/musli> for documentation.
//!
//! [Müsli web]: <https://docs.rs/musli-web>

use proc_macro::TokenStream;

mod define;

/// Define API types.
///
/// Defining an `endpoint` causes a type to be generated which is a marker type
/// for the endpoint, which binds together the request and response types.
///
/// Defining a broadcast simply associated a broadcast with a marker type.
///
/// The marker type is used with the various types used when interacting with an
/// API endpoint or broadcast, such as:
///
/// * [`web::Request`]
/// * [`web::Listener`]
///
/// These are in turn extended with the relevant API using them:
///
/// * [`yew022`] for yew `0.22.x`.
/// * [`yew023`] for yew `0.23.x`.
///
/// <br>
///
/// # Macro usage
///
/// The macro defines a set of endpoints and broadcasts, each of which is
/// represented by an uninhabitable type-level marker declared through a `type`
/// declaration.
///
/// On top of the API types, this macro also generates a `debug_id` function
/// with the following signature:
///
/// ```rust
/// use musli_web::api::MessageId;
///
/// fn debug_id(id: MessageId) -> impl core::fmt::Debug {
///     # "fake"
/// }
/// ```
///
/// This method can be used to debug a message id, unknown message ids will be
/// identified with an `Unknown(<number>)` debug printing.
///
/// Each type-level marker will implement either [`api::Endpoint`] or
/// [`api::Broadcast`]. And they will have an associated constant named `ID`
/// which matches the kind that are assocaited with them.
///
/// These roughly follow the structure of:
///
/// ```text
/// pub type <name>;
///
/// impl Endpoint for <name> {
///     <definition>
/// }
///
/// impl Broadcast for <name> {
///     <definition>
/// }
/// ```
///
/// Implementing an `Endpoint` can define requests and responses. The first
/// response defined is required and is the default response that certain APIs
/// will expected the endpoint to return. Any number of requests can be
/// specified, this is allows for different "sender types" to be defined, but
/// their over the wire format has to be the same.
///
/// Types specified as request types have to implement [`musli::Encode`] and
/// types sets as response types must implemente [`musli::Decode`].
///
/// ```text
/// (#[musli(..)])?
/// pub type Hello;
///
/// impl Endpoint for Hello (where <bounds>)? {
///     impl<'de> Request for HelloRequest<'de> (where <bounds>)?;
///     type Response<'de> = HelloResponse<'de> (where <bounds>)?;
/// }
/// ```
///
/// Implementing a `Broadcast` can define events, which are messages sent from
/// the server to the client. At least one event type is required, which will be
/// used as the default. Any number of events can be specified which allows for
/// different "sender types" to be defined, but their over the wire format has
/// to be the same.
///
/// ```text
/// (#[musli(..)])?
/// pub type Tick;
///
/// impl Broadcast for Tick (where <bounds>)? {
///     impl Event (<'lt>)? for OwnedTickEvent (where <bounds>)?;
/// }
/// ```
///
/// <br>
///
/// # Attributes
///
/// * `#[musli(kind = "...")]` - Explicitly sets the kind of an endpoint or
///   broadcast. Without it a string variant of the name of it will be used.
///
/// ```text
/// #[musli(kind = "tock")]
/// pub type Tick;
///
/// impl Broadcast for Tick {
///     impl Event for OwnedTickEvent;
///     impl<'de> Event for TickEvent<'de>;
/// }
/// ```
///
/// <br>
///
/// # Helpers enums
///
/// Each type declared within the macro becomes a variant inside of helper enums
/// that implements the [`api::Id`] trait.
///
/// If a type implements `Endpoint`, it becomes part of the `Request` enum. If
/// it implements `Broadcast` it becomes part of the `Event` enum.
///
///
/// ```
/// use musli::{Decode, Encode};
/// use musli_web::api::{self, Id};
///
/// #[derive(Encode, Decode)]
/// pub struct HelloRequest<'de> {
///     pub message: &'de str,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct HelloResponse<'de> {
///     pub message: &'de str,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct TickEvent<'de> {
///     pub message: &'de str,
///     pub tick: u32,
/// }
///
/// api::define! {
///     pub type Hello;
///
///     impl Endpoint for Hello {
///         impl<'de> Request for HelloRequest<'de>;
///         type Response<'de> = HelloResponse<'de>;
///     }
///
///     pub type Tick;
///
///     impl Broadcast for Tick {
///         impl<'de> Event for TickEvent<'de>;
///     }
/// }
///
/// fn main() {
///     let request = Request::from_id(Hello::ID);
///     assert_eq!(request, Request::Hello);
///
///     // Tick is not a request, so it becomes an unknown Request id.
///     let request = Request::from_id(Tick::ID);
///     assert_eq!(request, Request::Unknown(Tick::ID));
///
///     let event = Event::from_id(Tick::ID);
///     assert_eq!(event, Event::Tick);
///
///     // Hello is not a broadcast, so it becomes an unknown Event id.
///     let event = Event::from_id(Hello::ID);
///     assert_eq!(event, Event::Unknown(Hello::ID));
/// }
/// ```
///
/// <br>
///
/// # Including other lists
///
/// A list can include the list of another `define!` invocation, such as one
/// shipped by a library, so that the two can be carried on the same
/// connection:
///
/// ```text
/// include <path> as <Name>;
/// ```
///
/// `<path>` is the module the other `define!` was invoked in, and `<Name>`
/// names the variant it is included as in the `Request` and `Event` enums,
/// which wraps the `Request` and `Event` enums of the included list. The
/// marker types of the included list keep their ids and are used as they
/// are, by the client and the server alike. The variants are documented by
/// any doc comments on the `include`.
///
/// Every id of every list on a connection must be unique, since it is all
/// the client and the server use to tell messages apart. This is checked at
/// compile time: an id of the list which is also used by a list it includes,
/// or an id shared by two included lists, fails the build. Since a list
/// cannot see the ids of the lists which include it, a library list which is
/// meant to be included should pin its ids with `#[musli(id = ..)]` to a band
/// applications leave alone, since implicit ids count up from 1.
///
/// On the server, the composed `Request` enum is used as the [`ws::Handler::Id`]
/// as usual, and the requests of the included list are routed by matching on
/// its variant, such as by passing them on to a handler shipped with the
/// included list. A handler for the included list can instead be combined
/// with one for an application's own list (which does not include it) with
/// [`ws::Handler::or`].
///
/// ```
/// use musli::{Decode, Encode};
/// use musli_web::api::{self, Id};
///
/// /// A list shipped by a library.
/// mod auth {
///     use musli::{Decode, Encode};
///     use musli_web::api;
///
///     #[derive(Encode, Decode)]
///     pub struct LoginRequest;
///
///     #[derive(Encode, Decode)]
///     pub struct SessionEvent;
///
///     api::define! {
///         #[musli(id = 32000)]
///         pub type Login;
///
///         impl Endpoint for Login {
///             impl Request for LoginRequest;
///             type Response<'de> = bool;
///         }
///
///         #[musli(id = 32100)]
///         pub type Session;
///
///         impl Broadcast for Session {
///             impl Event for SessionEvent;
///         }
///     }
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct HelloRequest;
///
/// api::define! {
///     pub type Hello;
///
///     impl Endpoint for Hello {
///         impl Request for HelloRequest;
///         type Response<'de> = ();
///     }
///
///     /// The endpoints and broadcasts of the `auth` library.
///     include auth as Auth;
/// }
///
/// assert_eq!(Request::from_id(Hello::ID), Request::Hello);
/// assert_eq!(Request::from_id(auth::Login::ID), Request::Auth(auth::Request::Login));
/// assert_eq!(Event::from_id(auth::Session::ID), Event::Auth(auth::Event::Session));
/// assert_eq!(Request::Auth(auth::Request::Login).id(), auth::Login::ID);
///
/// assert!(Request::IDS.contains(auth::Login::ID));
/// assert_eq!(format!("{:?}", debug_id(auth::Login::ID)), "Auth(Login)");
/// ```
///
/// An id used by both lists fails to compile:
///
/// ```compile_fail
/// use musli::{Decode, Encode};
/// use musli_web::api;
///
/// mod auth {
///     # use musli::{Decode, Encode};
///     # use musli_web::api;
///     # #[derive(Encode, Decode)]
///     # pub struct LoginRequest;
///     api::define! {
///         // Implicitly `1`, just like `Hello` below.
///         pub type Login;
///
///         impl Endpoint for Login {
///             impl Request for LoginRequest;
///             type Response<'de> = bool;
///         }
///     }
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct HelloRequest;
///
/// api::define! {
///     pub type Hello;
///
///     impl Endpoint for Hello {
///         impl Request for HelloRequest;
///         type Response<'de> = ();
///     }
///
///     include auth as Auth;
/// }
/// ```
///
/// <br>
///
/// # Renamed dependencies
///
/// The code generated by the macro refers to `::musli_web`, so a crate which
/// depends on `musli-web` under another name (such as `musli-web04 = { package
/// = "musli-web", .. }`) would have to alias it back with `extern crate`. To
/// avoid that, the path of the crate to use can be given with an inner
/// attribute, which is the same `crate = <path>` as in `#[musli(crate =
/// ..)]`. It must precede everything else in the macro, and can be specified at
/// most once:
///
/// ```
/// # extern crate musli_web as musli_web04;
/// use musli::{Decode, Encode};
///
/// #[derive(Encode, Decode)]
/// pub struct HelloRequest;
///
/// musli_web04::api::define! {
///     #![musli(crate = musli_web04)]
///
///     pub type Hello;
///
///     impl Endpoint for Hello {
///         impl Request for HelloRequest;
///         type Response<'de> = ();
///     }
/// }
/// ```
///
/// The path is used verbatim to refer to the crate, so it can be anything that
/// resolves to it from where the macro is invoked, like `::musli_web04` or
/// `crate::reexports::musli_web`. Note that an `include`d list is unaffected,
/// since it is a path to the other list and not to the crate.
///
/// <br>
///
/// # Examples
///
/// ```
/// use musli::{Decode, Encode};
/// use musli_web::api;
///
/// #[derive(Encode, Decode)]
/// pub struct HelloRequest<'de> {
///     pub message: &'de str,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct HelloResponse<'de> {
///     pub message: &'de str,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct TickEvent<'de> {
///     pub message: &'de str,
///     pub tick: u32,
/// }
///
/// #[derive(Encode, Decode)]
/// pub struct OwnedTickEvent {
///     pub message: String,
///     pub tick: u32,
/// }
///
/// api::define! {
///     pub type Hello;
///
///     impl Endpoint for Hello {
///         impl<'de> Request for HelloRequest<'de>;
///         type Response<'de> = HelloResponse<'de>;
///     }
///
///     #[musli(id = 100)]
///     pub type Tick;
///
///     impl Broadcast for Tick {
///         impl Event for OwnedTickEvent;
///         impl<'de> Event for TickEvent<'de>;
///     }
/// }
///
/// assert_eq!(format!("{:?}", debug_id(Hello::ID)), "Hello");
/// assert_eq!(format!("{:?}", debug_id(Tick::ID)), "Tick");
/// ```
///
/// [`api::Id`]: <https://docs.rs/musli-web/latest/musli_web/api/trait.Id.html>
/// [`ws::Handler::Id`]: <https://docs.rs/musli-web/latest/musli_web/ws/trait.Handler.html#associatedtype.Id>
/// [`ws::Handler::or`]: <https://docs.rs/musli-web/latest/musli_web/ws/trait.Handler.html#method.or>
/// [`api::Broadcast`]: <https://docs.rs/musli-web/latest/musli_web/api/trait.Broadcast.html>
/// [`api::Endpoint`]: <https://docs.rs/musli-web/latest/musli_web/api/trait.Endpoint.html>
/// [`musli::Decode`]: <https://docs.rs/musli/latest/musli/trait.Decode.html>
/// [`musli::Encode`]: <https://docs.rs/musli/latest/musli/trait.Encode.html>
/// [`web::Listener`]: <https://docs.rs/musli-web/latest/musli_web/web/struct.Listener.html>
/// [`web::Request`]: <https://docs.rs/musli-web/latest/musli_web/web/struct.Request.html>
/// [`yew022`]: <https://docs.rs/musli-web/latest/musli_web/yew022/>
/// [`yew023`]: <https://docs.rs/musli-web/latest/musli_web/yew023/>
#[proc_macro]
pub fn define(input: TokenStream) -> TokenStream {
    let (path, input) = match define::split_crate(input.into()) {
        Ok(output) => output,
        Err(error) => return error.to_compile_error().into(),
    };

    let cx = define::cx(&path);

    let stream = define::expand(&cx, input);

    if let Some(stream) = cx.into_compile_errors() {
        return stream.into();
    };

    stream.into()
}
