use rand::{thread_rng, Rng};

const RANDOM_ALPHABET: &'static str = "ABCDEFGHIJKLMNOPQRSTTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";



pub fn generate_random_id(len: usize) -> String {
    let mut chars = vec![];

    let mut rng = thread_rng();

    for _ in 0..len {
        let x: usize = rng.gen_range(0..RANDOM_ALPHABET.len());

        chars.push(RANDOM_ALPHABET.as_bytes()[x])
    }

    String::from_utf8(chars).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::util::rand::generate_random_id;

    #[test]
    fn test_random_ids() {
        println!("{:?}", generate_random_id(5));
    }
}