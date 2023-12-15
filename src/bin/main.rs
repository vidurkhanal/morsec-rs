use morsec_rs::{MorsecStr, Parser};

fn main() {
    let m = MorsecStr::from("hello, world".into());

    let p = morsec_rs::map(morsec_rs::prefix("hello".into()), |x: String| {
        x.to_uppercase()
    })
    .parse(m);
    println!("{p:?}")
}
