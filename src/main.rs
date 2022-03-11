use ndarray::prelude::*;
use std::clone::Clone;
use std::fmt::Debug;

use num_traits::Num;

fn main() {
    let m1 = Matrix::from(array![[1, 2, 3], [4, 5, 6], [7, 8, 9]]);
    let m2 = Matrix::from(array![[1, 2], [3, 4], [5, 6]]);
    let m3 = match m1.mul(&m2) {
        Ok(m) => m,
        Err(e) => panic!("MatrixError: {}", e),
    };
    println!("m3 = {:#?}", &m3);
}

#[derive(Debug)]
struct Matrix<T> {
    shape: (usize, usize),
    container: Array2<T>,
}

impl<T: Num + Copy + Clone + Debug> Matrix<T> {
    fn new(shape: (usize, usize)) -> Matrix<T> {
        Matrix {
            shape,
            container: Array2::zeros(shape),
        }
    }

    fn from(eles: Array2<T>) -> Matrix<T> {
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

    fn mul(&self, other: &Matrix<T>) -> Result<Matrix<T>, String> {
        if self.n_cols() != other.n_rows() {
            Err(format!(
                "Shape {:?} and {:?} not aligned",
                self.shape, other.shape
            ))
        } else {
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
                                None => T::zero(),
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect();
            let container =
                Array2::<T>::from_shape_vec((self.n_rows(), other.n_cols()), data).unwrap();
            Ok(Matrix::<T>::from(container))
        }
    }
}
