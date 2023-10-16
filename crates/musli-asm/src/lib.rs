use musli_tests::models::Primitives;
use musli_zerocopy::{Buf, Error, OwnedBuf, Ref};

#[inline(never)]
pub fn musli_zerocopy_primitives_store(buf: &mut OwnedBuf, primitives: &Primitives) {
    buf.store(primitives);
}

#[inline(never)]
pub fn musli_zerocopy_primitives_load(
    buf: &Buf,
    reference: Ref<Primitives>,
) -> Result<&Primitives, Error> {
    buf.load(reference)
}
