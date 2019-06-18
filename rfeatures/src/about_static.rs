#[derive(Debug)]
struct Container<State> {
    state: State,
}

impl<State: 'static> Container<State> {
// impl<State> Container<State> {
    fn new(state: State) -> Container<State> {
        Container{state}
    }
}

#[derive(Debug)]
struct StateA {
    content: String,
}

#[derive(Debug)]
struct StateB<'a> {
    content: &'a str,
}

pub fn use_container() {
    let c_a = Container::new(StateA{content: "a".to_string()});

    // let b_string = "b".to_string();
    // let b = &b_string[..];
    // let c_b = Container::new(StateB{content: b}); // lifetime 不符合要求，编译不过去

    let c_b = Container::new(StateB{content: "b"});

    println!("c_a: {:?}", c_a);
    println!("c_b: {:?}", c_b);

}