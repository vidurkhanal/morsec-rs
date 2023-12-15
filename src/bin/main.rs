use morsec_rs::{MorsecStr, Parser};

fn main() {
    let m = MorsecStr::from("hello, world".into());

    let p = morsec_rs::prefix("hello".into()).parse(m);
    println!("{p:?}")
}
