//! Tests for the yew `0.22.x` handle context and hooks.
//!
//! Components are rendered on the server, so effects never run and no
//! connection is ever made. The handles used are not connected to a service.
//!
//! NB: Keep in sync with the tests for the other supported yew versions.

#![cfg(feature = "yew022")]

extern crate yew022 as yew;

use std::future::Future;

use musli_web::web03::prelude::*;
use musli_web::yew022::{
    ContextExt, HandleProvider, use_channel, use_handle, use_open, use_request,
};
use yew::LocalServerRenderer;
use yew::prelude::*;

/// Run a server side render to completion.
fn render<F>(future: F) -> String
where
    F: Future<Output = String>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let local = tokio::task::LocalSet::new();
    local.block_on(&runtime, future)
}

#[function_component(Hooks)]
fn hooks() -> Html {
    let handle = use_handle();
    let open = use_open();
    let channel = use_channel();
    let request = use_request();

    let same = request == request.clone();
    request.clear();

    html! {
        <p>
            {format!(
                "generation={} open={open:?} channel={} id={:?} pending={} same={same} handle={}",
                handle.generation(),
                channel.is_open(),
                channel.id(),
                request.is_pending(),
                *channel.handle() == handle,
            )}
        </p>
    }
}

#[derive(Properties, PartialEq)]
struct ProvideProps {
    children: Html,
}

#[function_component(Provide)]
fn provide(props: &ProvideProps) -> Html {
    html! {
        <HandleProvider context={ws::Handle::default()}>
            {props.children.clone()}
        </HandleProvider>
    }
}

#[test]
fn hooks_below_provider() {
    let html = render(
        LocalServerRenderer::<Provide>::with_props(ProvideProps {
            children: html!(<Hooks />),
        })
        .hydratable(false)
        .render(),
    );

    assert_eq!(
        html,
        "<p>generation=0 open=None channel=false id=NONE pending=false same=true handle=true</p>"
    );
}

struct StructComponent {
    handle: ws::Handle,
    found: bool,
}

impl Component for StructComponent {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            found: ctx.try_handle().is_some(),
            handle: ctx.handle(),
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html!(<p>{format!("found={} generation={}", self.found, self.handle.generation())}</p>)
    }
}

#[test]
fn context_ext_below_provider() {
    let html = render(
        LocalServerRenderer::<Provide>::with_props(ProvideProps {
            children: html!(<StructComponent />),
        })
        .hydratable(false)
        .render(),
    );

    assert_eq!(html, "<p>found=true generation=0</p>");
}

struct TryHandle;

impl Component for TryHandle {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html!(<p>{format!("found={}", ctx.try_handle().is_some())}</p>)
    }
}

#[test]
fn try_handle_without_provider() {
    let html = render(
        LocalServerRenderer::<TryHandle>::new()
            .hydratable(false)
            .render(),
    );

    assert_eq!(html, "<p>found=false</p>");
}

#[function_component(HandleOnly)]
fn handle_only() -> Html {
    let handle = use_handle();
    html!(<p>{handle.generation()}</p>)
}

#[test]
#[should_panic(
    expected = "no ws::Handle in context: wrap the component tree in musli_web::yew022::HandleProvider"
)]
fn use_handle_without_provider() {
    render(LocalServerRenderer::<HandleOnly>::new().render());
}

#[test]
#[should_panic(
    expected = "no ws::Handle in context: wrap the component tree in musli_web::yew022::HandleProvider"
)]
fn context_ext_without_provider() {
    render(LocalServerRenderer::<StructComponent>::new().render());
}
