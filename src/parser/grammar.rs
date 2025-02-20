use pest_consume::Parser as PP;

#[derive(PP)]
#[grammar = "parser/rules.pest"]
pub struct Parser;
