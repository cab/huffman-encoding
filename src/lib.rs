use std::{collections::HashMap, fmt, hash::Hash};

use bit_vec::BitVec;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No such key in encoding dictionary: {0}")]
    NoSuchKey(String),
    #[error("Invalid weight nodes: {0}")]
    InvalidWeights(String),
}

type Result<T> = std::result::Result<T, Error>;

struct Node<T> {
    freq: u32,
    value: Option<T>,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(freq: u32, value: Option<T>) -> Self {
        Self {
            freq,
            value,
            left: None,
            right: None,
        }
    }
}

pub struct Encoder<T> {
    encoding: HashMap<T, BitVec>,
}

impl<T> Encoder<T>
where
    T: Eq + Hash + Clone + fmt::Debug,
{
    fn new(root: &Node<T>) -> Self {
        fn assign<T>(node: &Node<T>, encoding: &mut HashMap<T, BitVec>, mut current_bits: BitVec)
        where
            T: Eq + Hash + Clone,
        {
            if let Some(ch) = node.value.as_ref() {
                encoding.insert(ch.clone(), current_bits);
            } else {
                if let Some(ref l) = node.left {
                    let mut bits = current_bits.clone();
                    bits.push(false);
                    assign(l, encoding, bits);
                }
                if let Some(ref r) = node.right {
                    current_bits.push(true);
                    assign(r, encoding, current_bits);
                }
            }
        }
        let mut encoding = HashMap::new();
        let bits = BitVec::new();
        assign(&root, &mut encoding, bits);
        Self { encoding }
    }

    pub fn encode(&self, data: &[T]) -> Result<BitVec> {
        let mut vec = BitVec::new();
        for item in data {
            let mut encoding = self
                .encoding
                .get(item)
                .ok_or_else(|| Error::NoSuchKey(format!("{:?}", item)))?
                .clone();
            vec.append(&mut encoding);
        }
        Ok(vec)
    }
}

pub struct Decoder<T> {
    root: Node<T>,
}

impl<T> Decoder<T> {
    fn new(root: Node<T>) -> Self {
        Self { root }
    }

    pub fn decode_iter<'a>(&'a self, encoded: &'a BitVec) -> impl Iterator<Item = &'a T> {
        DecoderIter {
            input: encoded.iter(),
            root: &self.root,
            current_node: &self.root,
        }
    }

    pub fn decode<'a>(&'a self, encoded: &'a BitVec) -> Vec<&'a T> {
        self.decode_iter(encoded).collect()
    }

    pub fn decode_owned(&self, encoded: &BitVec) -> Vec<T>
    where
        T: Clone,
    {
        self.decode_iter(encoded).cloned().collect()
    }
}

struct DecoderIter<'a, T> {
    root: &'a Node<T>,
    input: bit_vec::Iter<'a>,
    current_node: &'a Node<T>,
}

impl<'a, T> Iterator for DecoderIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let bit = self.input.next()?;
        if bit {
            if let Some(ref right) = self.current_node.right {
                self.current_node = right;
            }
        } else if let Some(ref left) = self.current_node.left {
            self.current_node = left;
        }
        if let Some(value) = self.current_node.value.as_ref() {
            self.current_node = &self.root;
            Some(value)
        } else {
            self.next()
        }
    }
}

pub struct Huffman<T> {
    encoder: Encoder<T>,
    decoder: Decoder<T>,
}

impl<T> Huffman<T>
where
    T: Eq + Hash + Clone + fmt::Debug,
{
    pub fn new(weights: impl IntoIterator<Item = (T, u32)>) -> Result<Self> {
        let mut nodes = weights
            .into_iter()
            .map(|(value, frequency)| Box::new(Node::new(frequency, Some(value))))
            .collect::<Vec<_>>();

        while nodes.len() > 1 {
            nodes.sort_by(|a, b| (&(b.freq)).cmp(&(a.freq)));
            let a = nodes
                .pop()
                .ok_or_else(|| Error::InvalidWeights("Expected at least 1 node".to_string()))?;
            let b = nodes
                .pop()
                .ok_or_else(|| Error::InvalidWeights("Expected at least 1 node".to_string()))?;
            let mut c = Node::new(a.freq + b.freq, None);
            c.left = Some(a);
            c.right = Some(b);
            nodes.push(Box::new(c));
        }

        let root = *nodes
            .pop()
            .ok_or_else(|| Error::InvalidWeights("Expected root node".to_string()))?;
        let encoder = Encoder::new(&root);
        let decoder = Decoder::new(root);
        Ok(Self { encoder, decoder })
    }

    /// Encodes data into a BitVec. Fails if any of the data is not present in the dictionary.
    pub fn encode(&self, data: &[T]) -> Result<BitVec> {
        self.encoder.encode(data)
    }

    pub fn decode<'a>(&'a self, encoded: &'a BitVec) -> Vec<&'a T> {
        self.decoder.decode(encoded)
    }

    pub fn decode_iter<'a>(&'a self, encoded: &'a BitVec) -> impl Iterator<Item = &'a T> {
        self.decoder.decode_iter(encoded)
    }

    pub fn decode_owned(&self, encoded: &BitVec) -> Vec<T> {
        self.decoder.decode_owned(encoded)
    }

    /// Split into a Encoder and Decoder.
    pub fn split(self) -> (Encoder<T>, Decoder<T>) {
        (self.encoder, self.decoder)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_encode_decode_i32() {
        let weights = vec![(0, 10), (1, 1), (2, 5)];
        let huffman = Huffman::new(weights).unwrap();
        let data = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let encoded = huffman.encode(&data).unwrap();
        let decoded = huffman.decode_owned(&encoded);
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_split() {
        let weights = vec![(0, 10), (1, 1), (2, 5)];
        let (encoder, decoder) = Huffman::new(weights).unwrap().split();
        let data = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        let encoded = encoder.encode(&data).unwrap();
        let decoded = decoder.decode_owned(&encoded);
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_encode_decode_string() {
        let weights = vec![
            ("hello".to_string(), 2),
            ("hey".to_string(), 3),
            ("howdy".to_string(), 1),
        ];
        let huffman = Huffman::new(weights).unwrap();
        let data = vec!["howdy".into(), "howdy".into(), "hey".into(), "hello".into()];
        let encoded = huffman.encode(&data).unwrap();
        let decoded = huffman.decode_owned(&encoded);
        assert_eq!(data, decoded);
    }
}
