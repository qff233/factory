pub struct Recipe {
    id: i32,
    tool_type: String,
    name: String,
    version: String,
    status: String,
    inputs: Vec<String>,
    inputbuss: Vec<String>,
    created_by: String,
    created_at: String,
    updated_at: String,
}
