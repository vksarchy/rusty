fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_adds() {
        assert_eq!(2 + 2, 4);
    }
}
