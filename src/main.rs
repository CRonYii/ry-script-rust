mod rymath;

pub use crate::rymath::matrix::*;

fn main() {
    let m1 = matrix![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    let m2 = matrix![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
    let m3 = m1 * m2;
    println!("m3:\n{}", &m3);
}