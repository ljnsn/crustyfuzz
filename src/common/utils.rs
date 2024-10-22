pub fn is_none<T>(s: Option<&[T]>) -> bool {
    match s {
        Some(slice) => slice.is_empty(),
        None => true,
    }
}
