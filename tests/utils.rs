use sunny::utils;

#[test]
fn timestamp_case1() {
    let ret = utils::timestamp("28 Sep 2014 04:19:31 GMT");

    assert!(ret.is_some());
}

#[test]
fn timestamp_case2() {
    let ret = utils::timestamp("released September 28, 2014");

    assert!(ret.is_some());
}
