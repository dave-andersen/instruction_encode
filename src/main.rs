// Quick hack huffman instruction encoding
// lots of code stolen from https://aufdj.gitbook.io/data-compression-with-rust/huffman-encoding

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::collections::{BinaryHeap, BTreeMap};
use std::cmp::Ord;
use std::cmp::Ordering;
use anyhow::Result;
use serde::Deserialize;
use std::io::BufReader;


fn main() {
    if let Err(e) = create_instruction_mapping() {
        println!("Error: {e}");
    }
}
#[derive(Deserialize, Debug)]
struct InstructionDef {
    name: String,
    max_len: i32
}

#[derive(Eq, PartialEq, Debug)]
enum NodeType {
    Internal(Box<Node>, Box<Node>),
    Leaf(String)
}

#[derive(Debug)]
struct Node {
    probability: f32,
    node_type: NodeType,
}

impl Node {
    fn new(probability: f32, node_type: NodeType) -> Node {
        Node { probability: probability, node_type }
    }
}

impl Eq for Node { }

impl PartialEq for Node {
    fn eq(&self, rhs: &Self) -> bool {
        self.node_type == rhs.node_type
    }
}

impl Ord for Node {
    fn cmp(&self, rhs: &Self) -> Ordering {
        if self == rhs {
            return Ordering::Equal;
        }
        rhs.probability.partial_cmp(&self.probability).unwrap()
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        rhs.probability.partial_cmp(&self.probability)
    }
}

type HuffmanCodeMap = BTreeMap<String, Vec<u8>>;

fn gen_codes(node: &Node, prefix: Vec<u8>, codes: &mut HuffmanCodeMap) {
    match &node.node_type {
        NodeType::Internal(ref left_child, ref right_child) => {
            let mut left_prefix = prefix.clone();
            left_prefix.push(0);
            gen_codes(&left_child, left_prefix, codes);

            let mut right_prefix = prefix;
            right_prefix.push(1);
            gen_codes(&right_child, right_prefix, codes);
        }
        NodeType::Leaf(name) => {
            codes.insert(name.to_string(), prefix);
        }
    }
}

fn build_tree(heap: &mut BinaryHeap<Node>) {
    while heap.len() > 1 {
        let left_child = heap.pop().unwrap();
        let right_child = heap.pop().unwrap();
        heap.push(
            Node::new(
                left_child.probability + right_child.probability, 
                NodeType::Internal(Box::new(left_child), Box::new(right_child))
            )
        );
    }
}

fn create_instruction_mapping() -> Result<()> {
    let file = File::open("instructions.json")?;
    let reader = BufReader::new(file);
    let instructions: Vec<InstructionDef> = serde_json::from_reader(reader)?;
    //println!("Instructions\n{:?}", instructions);

    let mut heap: BinaryHeap<Node> = BinaryHeap::new();
    for i in instructions {
        heap.push(                                          
            Node::new(1.0 / ((1<<i.max_len) as f32), NodeType::Leaf(i.name))
        );                                  
    }
    
    //println!("Starting Heap: {:?}", heap);
    //println!("First entry in heap: {:?}", heap.peek());
    build_tree(&mut heap);
    //println!("Final Heap: {:?}", heap);

    let mut codes = HuffmanCodeMap::new();  
    gen_codes(heap.peek().unwrap(), vec![0u8; 0], &mut codes);
    for c in codes {
        println!("{} {:?}", c.0, c.1.iter().map(|x| x.to_string()).collect::<String>());
    }
    Ok(())
}