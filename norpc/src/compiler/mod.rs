mod generator;
mod parser;

#[derive(Debug)]
struct Service {
    name: String,
    functions: Vec<Function>,
}
#[derive(Debug)]
struct Function {
    name: String,
    inputs: Vec<Parameter>,
    output: String,
}
#[derive(Debug)]
struct Parameter {
    var_name: String,
    typ_name: String,
}
pub(crate) fn compile(s: &str) -> String {
    let mut s = s.to_owned();
    s.retain(|c| !c.is_whitespace());
    dbg!(&s);
    let services = parser::parse(&s).unwrap().1;
    dbg!(&services);
    generator::generate(services)
}
