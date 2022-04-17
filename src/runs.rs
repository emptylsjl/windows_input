

// pub fn type_of<T>(_: &T) -> &'static str {
pub fn var_type<T>(_: &T) -> String {
    std::intrinsics::type_name::<T>().to_string()
}