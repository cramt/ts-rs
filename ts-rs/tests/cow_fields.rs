#![allow(dead_code)]

use std::borrow::Cow;
use ts_rs::TS;

#[derive(TS)]
pub struct CompressedFit<'a> {
    pub a: Cow<'a, str>,
    pub b: Cow<'a, str>,
    pub c: Vec<Cow<'a, str>>,
    pub d: Vec<Cow<'a, str>>,
    pub e: Vec<Cow<'a, str>>,
}


#[test]
fn test() {
    assert_eq!(
        CompressedFit::inline(0),
        "\
{
    a: string,
    b: string,
    c: string[],
    d: string[],
    e: string[],
}"
    )
}
