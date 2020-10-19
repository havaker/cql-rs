pub struct Query {
    query_text: String,
}

impl Query {
    pub fn new(query_text: String) -> Query {
        return Query { query_text };
    }

    pub fn get_query_text(&self) -> String {
        return self.query_text.clone();
    }
}
