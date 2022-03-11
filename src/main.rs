use ndarray::prelude::*;
use std::fmt;
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
    println!("m3:\n{}", &m3);
}

struct Matrix {
    shape: (usize, usize),
    container: Array2<f64>,
}

/* the ndarray module itself can already perform matrix opeartion just like numpy.
 * Implementing the matrix class is just for my own learning purpose.
 */
impl Matrix {
    fn new(shape: (usize, usize)) -> Matrix {
        Matrix {
            shape,
            container: Array2::zeros(shape),
        }
    }

    fn from(eles: Array2<f64>) -> Matrix {
        let dim = eles.shape();
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

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.container)
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
        let n_rows = self.n_rows();
        let n_cols = other.n_cols();
        let mut container = Array2::<f64>::zeros((n_rows, n_cols));
        for r in 0..n_rows {
            for c in 0..n_cols {
                for x in 0..self.n_cols() {
                    container[[r, c]] += self.container[[r, x]] * other.container[[x, c]];
                }
            }
        }
        Matrix::from(container)
    }
}
