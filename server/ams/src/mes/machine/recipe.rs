struct Item {
    name: String,
    count: i32,
}

pub struct Recipe {
    inputs: Vec<Item>,
    inputbuss: Vec<Item>,
}
