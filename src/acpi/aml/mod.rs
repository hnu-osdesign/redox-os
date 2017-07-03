//! # AML
//! Code to parse and execute AML tables

use alloc::boxed::Box;
use collections::string::String;
use collections::vec::Vec;
use collections::btree_map::BTreeMap;
use core::fmt::Debug;
use core::str::FromStr;

use super::sdt::Sdt;

#[macro_use]
mod parsermacros;

mod namespace;
mod termlist;
mod namespacemodifier;
mod pkglength;
mod namestring;
mod namedobj;
mod dataobj;
mod type1opcode;
mod type2opcode;
mod parser;

use self::parser::{ ParseResult, AmlExecutionContext };
use self::termlist::parse_term_list;
pub use self::namespace::AmlValue;

#[derive(Debug)]
pub enum AmlError {
    AmlParseError(&'static str),
    AmlInvalidOpCode,
    AmlValueError,
    AmlDeferredLoad,
    AmlFatalError(u8, u16, AmlValue)
}

pub fn parse_aml_table(sdt: &'static Sdt) -> Result<BTreeMap<String, AmlValue>, AmlError> {
    let data = sdt.data();
    let mut ctx = AmlExecutionContext::new(String::from_str("\\").unwrap());
    
    let term_list = parse_term_list(data, &mut ctx)?;

    Ok(ctx.namespace.clone())
}

pub fn is_aml_table(sdt: &'static Sdt) -> bool {
    if &sdt.signature == b"DSDT" || &sdt.signature == b"SSDT" {
        true
    } else {
        false
    }
}
