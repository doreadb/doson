pub mod value;

#[cfg(test)]
mod tests {
    use crate::value;

    #[test]
    fn it_works() {
        let v = value::DataValue::from("(1,2)");
        println!("{:?}", v)
    }
}
