use infra_bits::rand::RTreeMap;

#[derive(Debug)]
struct Dropcheck {
    inner: char,
}

static mut DROPPED: usize = 0;

impl Drop for Dropcheck {
    fn drop(&mut self) {
        println!("{}", self.inner);
        unsafe { DROPPED += 1 };
    }
}
#[test]
fn test_should_dropp_all_elements() {
    {
        let mut tree: RTreeMap<char, Dropcheck> = RTreeMap::new();
        for c in 'a'..='e' {
            tree.insert(c, Dropcheck { inner: c })
        }
        for c in ('a'..='e').into_iter().rev() {
            println!("before delete: {:?}", tree);
            tree.remove(c);
            println!("after delete: {:?}", tree);
            // assert!(deleted.unwrap().inner == c)
        }
    }
    assert_eq!(unsafe { DROPPED }, 5);
}
