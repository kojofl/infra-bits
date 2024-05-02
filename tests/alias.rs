use infra_bits::rand::{Alias, EventEmmiter};

#[test]
#[should_panic]
fn test_should_error_on_non_uniform() {
    let _alias = Alias::new(&[0.333, 0.333, 0.333]);
}

#[test]
fn test_generate() {
    let alias = Alias::new(&[0.08, 0.12, 0.10, 0.07, 0.0, 0.28, 0.11, 0.08, 0.06, 0.10]);
    println!("{:?}", alias)
}

#[test]
fn test_generate_event() {
    let alias: EventEmmiter<3, Box<dyn Fn(&mut [usize])>> = EventEmmiter::new(
        &[0.2, 0.3, 0.5],
        [
            Box::new(event_one),
            Box::new(event_two),
            Box::new(event_three),
        ],
    );
    let mut res = [0, 0, 0];
    for _ in 0..1000000 {
        let event = alias.generate();
        event(&mut res);
    }
    assert!(res[0] < res[1]);
    assert!(res[1] < res[2]);
}

fn event_one(s: &mut [usize]) {
    s[0] += 1;
}

fn event_two(s: &mut [usize]) {
    s[1] += 1;
}

fn event_three(s: &mut [usize]) {
    s[2] += 1;
}
