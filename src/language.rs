use tree_sitter::{Parser, Query};

pub struct Language {
    highlights: Query,
    parser: Parser,
}

impl Language {
    pub fn highlights(&self) -> &Query {
        &self.highlights
    }

    pub fn parser(&mut self) -> &mut Parser {
        &mut self.parser
    }
}

pub fn rust() -> Language {
    let lang = tree_sitter_rust::language();

    let highlights = Query::new(&lang, tree_sitter_rust::HIGHLIGHTS_QUERY).unwrap();

    let mut parser = Parser::new();
    parser.set_language(&lang).unwrap();

    Language { highlights, parser }
}
