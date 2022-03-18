use super::*;

#[test]
fn matrix_mul() {
    let m1 = matrix![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    let m2 = matrix![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
    let m3 = m1 * m2;
    assert_eq!(m3, matrix![[22.0, 28.0], [49.0, 64.0], [76.0, 100.0]]);
}
