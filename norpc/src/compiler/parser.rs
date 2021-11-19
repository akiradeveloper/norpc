//! File = Service+
//! Service = service Name { Def+ }
//! Name = String
//! Def = "fn" FunName ( Parameter+ ) -> Type ;
//! FunName = String
//! Type = String
//! Var = String
//! Parameter = Var : Type

use super::*;

use nom::branch::alt;
use nom::bytes::complete::{is_a, tag, take_while1};
use nom::character::complete::{alphanumeric1, char, one_of};
use nom::character::{is_alphanumeric, is_digit};
use nom::combinator::{all_consuming, map};
use nom::multi::{many0, many1, many_m_n, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;

macro_rules! tt {
    ($p: ident, $s: expr) => {
        assert!(all_consuming($p)($s).is_ok())
    };
}

fn parse_typeident(s: &str) -> IResult<&str, String> {
    let ident = is_a("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_");
    map(ident, |x: &str| x.to_owned())(s)
}
fn parse_typename2(s: &str) -> IResult<&str, String> {
    let array = delimited(
        tag("["),
        separated_list1(tag(","), parse_typename2),
        tag("]"),
    );
    let array = map(array, |xs| {
        let mut out = String::new();
        out.push('[');
        out.push_str(&itertools::join(xs, ","));
        out.push(']');
        out
    });
    let tuple = delimited(
        tag("("),
        separated_list0(tag(","), parse_typename2),
        tag(")"),
    );
    let tuple = map(tuple, |xs| {
        let mut out = String::new();
        out.push('(');
        out.push_str(&itertools::join(xs, ","));
        out.push(')');
        out
    });
    let composite = pair(
        parse_typeident,
        delimited(
            tag("<"),
            separated_list1(tag(","), parse_typename2),
            tag(">"),
        ),
    );
    let composite = map(composite, |(a, xs)| {
        let mut out = String::new();
        out.push_str(&a);
        out.push('<');
        out.push_str(&itertools::join(xs, ","));
        out.push('>');
        out
    });
    alt((array, tuple, composite, parse_typeident))(s)
}
#[test]
fn test_typename2() {
    tt!(parse_typename2, "u64");
    tt!(parse_typename2, "HashSet<u64>");
    tt!(parse_typename2, "()");
    tt!(parse_typename2, "HashSet<(u64,u64)>");
    tt!(parse_typename2, "HashSet<((),Vec<u8>)>");
}
// fn parse_typename(s: &str) -> IResult<&str, String> {
//     let p1 = is_a("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789<>_");
//     let p = alt((p1, tag("()")));
//     map(p, |x: &str| x.to_owned())(s)
// }
// #[test]
// fn test_typename() {
//     assert!(parse_typename("u64").is_ok());
//     assert!(parse_typename("HashSet<u64>").is_ok());
//     assert!(parse_typename("()").is_ok());
// }
fn parse_varname(s: &str) -> IResult<&str, String> {
    let p = is_a("abcdefghijklmnopqrstuvwxyz0123456789_");
    map(p, |x: &str| x.to_owned())(s)
}
#[test]
fn test_varname() {
    tt!(parse_varname, "x");
    tt!(parse_varname, "x0");
    tt!(parse_varname, "n_n");
}
fn parse_param(s: &str) -> IResult<&str, Parameter> {
    let p1 = parse_varname;
    let p2 = parse_typename2;
    let p = separated_pair(p1, tag(":"), p2);
    map(p, |(x, y)| Parameter {
        var_name: x.to_owned(),
        typ_name: y.to_owned(),
    })(s)
}
#[test]
fn test_parse_param() {
    tt!(parse_param, "x:u64");
    tt!(parse_param, "n_n:u64");
    tt!(parse_param, "x:HashSet<u64>");
}
fn parse_function(s: &str) -> IResult<&str, Function> {
    let p1 = preceded(tag("fn"), parse_varname);
    let p2 = delimited(tag("("), separated_list0(tag(","), parse_param), tag(")"));
    let p3 = preceded(tag("->"), parse_typename2);
    map(terminated(tuple((p1, p2, p3)), tag(";")), |(x, y, z)| {
        Function {
            name: x.to_owned(),
            inputs: y,
            output: z.to_owned(),
        }
    })(s)
}
#[test]
fn test_parse_function() {
    tt!(parse_function, "fnadd(s:String)->i32;");
    tt!(parse_function, "fnadd_one(s:String)->();");
    tt!(parse_function, "fnadd(s:String)->HashSet<u64>;");
    tt!(parse_function, "fnwrite(a:u64,b:u64)->u64;");
    tt!(parse_function, "fnwrite(id:u64,s:String)->();");
}
fn parse_functions(s: &str) -> IResult<&str, Vec<Function>> {
    let p1 = tag("{");
    let p2 = many1(parse_function);
    let p3 = tag("}");
    delimited(p1, p2, p3)(s)
}
#[test]
fn test_functions() {
    tt!(parse_functions, "{fnadd(s:String)->i32;}");
    tt!(
        parse_functions,
        "{fnadd(s:String)->i32;fnadd(s:String)->();}"
    );
}
fn parse_service(s: &str) -> IResult<&str, Service> {
    let p1 = preceded(tag("service"), parse_typename2);
    let p = pair(p1, parse_functions);
    map(p, |(name, functions)| Service {
        name: name.to_owned(),
        functions,
    })(s)
}
#[test]
fn test_service() {
    tt!(
        parse_service,
        "serviceHello{fnread(id:u64)->Option<String>;fnwrite(id:u64,s:String)->();}"
    );
}
pub(super) fn parse(s: &str) -> IResult<&str, Vec<Service>> {
    all_consuming(many1(parse_service))(s)
}
