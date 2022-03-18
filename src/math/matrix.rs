use ndarray::prelude::*;
use std::fmt;
use std::ops;

#[derive(Debug, PartialEq)]
pub struct Matrix {
    shape: (usize, usize),
    container: Array2<f64>,
}

#[macro_export]
macro_rules! matrix {
    ($([$($x:expr),* $(,)*]),+ $(,)*) => {{
        Matrix::from(ndarray::array![$([$($x,)*],)*])
    }};
}

/* the ndarray module itself can already perform matrix opeartion just like numpy.
 * Implementing the matrix class is just for my own learning purpose.
 */
impl Matrix {
    pub fn from(eles: Array2<f64>) -> Matrix {
        let dim = eles.shape();
        let shape = (dim[0], dim[1]);
        Matrix {
            shape,
            container: eles,
        }
    }

    pub fn n_rows(&self) -> usize {
        self.shape.0
    }

    pub fn n_cols(&self) -> usize {
        self.shape.1
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut max = 0.0;
        for x in self.container.iter() {
            if *x > max {
                max = *x;
            }
        }
        let mut max_int = max as i64;
        let mut n_digit = 0;
        while max_int != 0 {
            n_digit += 1;
            max_int /= 10;
        }
        for r in 0..self.n_rows() {
            if r != 0 {
                write!(f, "\n [")?
            } else {
                write!(f, "[[")?
            }
            for c in 0..self.n_cols() {
                if c != 0 {
                    write!(f, " ")?
                }
                write!(f, "{:1$}", self.container[[r, c]], n_digit)?
            }
            write!(f, "]")?
        }
        write!(f, "]")
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
