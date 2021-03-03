#![allow(dead_code)]

use ts_rs::TS;

#[derive(TS)]
struct BorrowedValues<'a, 'b> {
    a: &'a str,
    b: &'b str,
}

#[test]
fn test() {
    assert_eq!(
        BorrowedValues::inline(0),
        "\
{
    a: string,
    b: string,
}"
    )
}
