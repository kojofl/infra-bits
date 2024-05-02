use infra_bits::rand::RTree;

#[test]
fn test_should_be_able_to_fill_tree() {
    let mut tree = RTree::new();
    tree.insert('b');
    println!("{:?}", unsafe { tree.root.unwrap().as_ref() });
    tree.insert('z');
    println!("{:?}", unsafe { tree.root.unwrap().as_ref() });
    tree.insert_high_prio('a');
    println!("{:?}", unsafe { tree.root.unwrap().as_ref() });
}
