//! Example showcases how a [DAG] can be encoded and accessed.
//!
//! This encodes the same DAG which is visible on the Wikipedia page linked, at
//! time of writing it looked like this:
//!
//! ```text
//! a → b
//! a → d
//! a → c
//! a → e
//! b → d
//! c → d
//! c → e
//! d → e
//! ```
//!
//! [DAG]: <https://en.wikipedia.org/wiki/Directed_acyclic_graph>

use anyhow::{Context, Error};
use musli_zerocopy::{OwnedBuf, Ref, ZeroCopy};

#[derive(ZeroCopy)]
#[repr(C)]
struct Node {
    id: char,
    nodes: Ref<[Ref<Node>]>,
}

fn store_dag(buf: &mut OwnedBuf) -> Ref<Node> {
    let a = buf.store_uninit::<Node>();
    let b = buf.store_uninit::<Node>();
    let c = buf.store_uninit::<Node>();
    let d = buf.store_uninit::<Node>();
    let e = buf.store_uninit::<Node>();

    let nodes = buf.store_slice::<Ref<Node>>(&[]);
    buf.load_uninit_mut(e).write(&Node { id: 'e', nodes });
    let e = e.assume_init();

    let nodes = buf.store_slice(&[e]);
    buf.load_uninit_mut(d).write(&Node { id: 'd', nodes });
    let d = d.assume_init();

    let nodes = buf.store_slice(&[d, e]);
    buf.load_uninit_mut(c).write(&Node { id: 'c', nodes });
    let c = c.assume_init();

    let nodes = buf.store_slice(&[d]);
    buf.load_uninit_mut(b).write(&Node { id: 'b', nodes });
    let b = b.assume_init();

    let nodes = buf.store_slice(&[b, c, e]);
    buf.load_uninit_mut(a).write(&Node { id: 'a', nodes });
    a.assume_init()
}

fn main() -> Result<(), Error> {
    let mut buf = OwnedBuf::new();

    let root = store_dag(&mut buf);

    let root = buf.load(root)?;

    let b = buf.load(root.nodes.get(0).context("missing node")?)?;
    let b = buf.load(*b)?;

    let d = buf.load(b.nodes.get(0).context("missing node")?)?;
    let d = buf.load(*d)?;

    dbg!(root.id, root.nodes.len());
    dbg!(b.id, b.nodes.len());
    dbg!(d.id, d.nodes.len());

    Ok(())
}
