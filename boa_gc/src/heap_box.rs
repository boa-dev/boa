

pub(crate) struct HeapBoxHeader;


pub(crate) struct HeapBox<T: Trace + 'static> {
    header: HeapBoxHeader,
    object: T,
}