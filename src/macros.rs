macro_rules! repeat_enum_variant {
    ($current_value:expr) => {
        SecondaryReference = $current_value,
        $(SecondaryReference = $current_value+1,)*
    };
}