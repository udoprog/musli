use musli::{Decode, Encode};
use std::marker::PhantomData;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Mesh<V: AsRef<[u32]> = Vec<u32>> {
    pub triangles: V,
}

#[derive(Debug, Clone, Encode, Decode)]
#[musli(Binary, bound = {T}, decode_bound = {T})]
pub struct Ignore<T> {
    #[musli(default)]
    pub _marker: PhantomData<T>,
}
