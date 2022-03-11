use ndarray::prelude::*;
use std::fmt::Debug;
use std::ops;

#[macro_export]
macro_rules! matrix {
    ($([$($x:expr),* $(,)*]),+ $(,)*) => {{
        $crate::Matrix::from(array![$([$($x,)*],)*])
    }};
}

fn main() {
    let m1 = matrix![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    let m2 = matrix![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
    let m3 = m1 * m2;
    // let m3 = match m1.mul(&m2) {
    //     Ok(m) => m,
    //     Err(e) => panic!("MatrixError: {}", e),
    // };
    println!("m3 = {:#?}", &m3);
}

#[derive(Debug)]
struct Matrix {
    shape: (usize, usize),
    container: Array2<f64>,
}

impl Matrix {
    fn new(shape: (usize, usize)) -> Matrix {
        Matrix {
            shape,
            container: Array2::zeros(shape),
        }
    }

    fn from(eles: Array2<f64>) -> Matrix {
        let dim = eles.raw_dim();
        let shape = (dim[0], dim[1]);
        Matrix {
            shape,
            container: eles,
        }
    }

    fn n_rows(&self) -> usize {
        self.shape.0
    }

    fn n_cols(&self) -> usize {
        self.shape.1
    }

}

impl ops::Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        if self.n_cols() != other.n_rows() {
            panic!(
                "Matrix multiplication error: shape {:?} and {:?} not aligned",
                self.shape, other.shape
            )
        }
        let data = self
            .container
            .axis_iter(Axis(0))
            .map(|row| {
                other
                    .container
                    .axis_iter(Axis(1))
                    .map(|col| {
                        match row
                            .iter()
                            .zip(col.iter())
                            .map(|(a, b)| (*a) * (*b))
                            .reduce(|accum, i| accum + i)
                        {
                            Some(r) => r,
                            None => 0.0,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();
        let container =
            Array2::<f64>::from_shape_vec((self.n_rows(), other.n_cols()), data).unwrap();
        Matrix::from(container)
    }
}
