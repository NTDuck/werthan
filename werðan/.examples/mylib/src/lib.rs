pub fn add(left: ::core::primitive::u64, right: ::core::primitive::u64) -> ::core::primitive::u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        ::std::assert_eq!(result, 4);
    }
}
