pub fn is_none<T>(s: Option<&[T]>) -> bool {
    // TODO: check for nan
    match s {
        Some(_) => false,
        None => true,
    }
}
