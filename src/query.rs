pub struct Query {
    query_text: String,
}

impl Query {
    pub fn new(query_text: &str) -> Query {
        return Query {
            query_text: query_text.to_string(),
        };
    }

    pub fn get_query_text(&self) -> String {
        return self.query_text.clone();
    }
}
