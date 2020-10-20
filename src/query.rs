pub struct Query {
    query_text: &'static str,
}

impl Query {
    pub fn new(query_text: &'static str) -> Query {
        return Query { query_text };
    }

    pub fn get_query_text(&self) -> &'static str {
        return self.query_text.clone();
    }
}
