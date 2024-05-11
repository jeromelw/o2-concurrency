use anyhow::{anyhow, Result};
use std::{
    fmt,
    ops::{Add, AddAssign, Mul},
    sync::mpsc,
    thread,
};

use crate::{dot_product, Vector};

// [[1, 2], [3, 4], [5, 6] * [[1, 2, 3], [4, 5, 6]] = [[9, 12, 15], [19, 26, 33], [29, 40, 51]]

pub struct Matrix<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T: fmt::Debug> Matrix<T> {
    pub fn new(rows: usize, cols: usize, data: impl Into<Vec<T>>) -> Self {
        Self {
            rows,
            cols,
            data: data.into(),
        }
    }
}

impl<T> fmt::Display for Matrix<T>
where
    T: fmt::Display,
{
    //display a 2X3 as {1 2 3, 4 5 6}, 3X2 as {1 2, 3 4, 5 6}
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.rows {
            for j in 0..self.cols {
                write!(f, "{}", self.data[i * self.cols + j])?;
                if j != self.cols - 1 {
                    write!(f, " ")?;
                }
            }
            if i != self.rows - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;

        Ok(())
    }
}

impl<T> fmt::Debug for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Matrix(rows={}, cols={}): {}",
            self.rows, self.cols, self
        )
    }
}

impl<T> Mul for Matrix<T>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T> + fmt::Debug + Send + 'static,
{
    type Output = Matrix<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        multiply_matrix(&self, &rhs).expect("Matrix multiply error")
    }
}

struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

impl<T> MsgInput<T> {
    fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

#[derive(Debug)]
struct MsgOutput<T> {
    idx: usize,
    value: T,
}

impl<T> MsgOutput<T> {
    fn new(idx: usize, value: T) -> Self {
        Self { idx, value }
    }
}

struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>,
}

impl<T> Msg<T> {
    fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

const NUM_THREADS: usize = 4;

pub fn multiply_matrix<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T> + fmt::Debug + Send + 'static,
{
    if a.cols != b.rows {
        return Err(anyhow!("Matrix dimensions do not match"));
    }
    let matrix_len = a.rows * b.cols;

    let mut data = vec![T::default(); matrix_len];

    let senders = (0..NUM_THREADS)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();

            thread::spawn(move || {
                for msg in rx {
                    let value = dot_product(msg.input.row, msg.input.col)?;
                    //println!("dot_product value: {:?}", value);
                    let output = MsgOutput::new(msg.input.idx, value);
                    //println!("output: {:?}", output);

                    if let Err(e) = msg.sender.send(output) {
                        eprintln!("Error sending ret message: {:?}", e);
                    }
                }
                Ok::<_, anyhow::Error>(())
            });
            tx
        })
        .collect::<Vec<_>>();
    let mut receivers = Vec::with_capacity(matrix_len);

    for i in 0..a.rows {
        for j in 0..b.cols {
            let row = Vector::new(&a.data[i * a.cols..(i + 1) * a.cols]);
            let col = Vector::new(
                b.data[j..]
                    .iter()
                    .step_by(b.cols)
                    .copied()
                    .collect::<Vec<_>>(),
            );
            let idx = i * b.cols + j;
            let input = MsgInput::new(idx, row, col);

            let (tx, rx) = oneshot::channel();

            let msg = Msg::new(input, tx);
            if let Err(e) = senders[i % NUM_THREADS].send(msg) {
                eprintln!("Error sending param message: {:?}", e);
            }
            receivers.push(rx);
        }
    }

    for rx in receivers {
        let output = rx.recv()?;
        data[output.idx] = output.value;
    }

    Ok(Matrix::new(a.rows, b.cols, data))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_matrix_multiplication() -> Result<()> {
        let a = Matrix::new(3, 2, [1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(2, 3, [1, 2, 3, 4, 5, 6]);
        let c = a * b;
        assert_eq!(c.rows, 3);
        assert_eq!(c.cols, 3);
        assert_eq!(c.data, vec![9, 12, 15, 19, 26, 33, 29, 40, 51]);
        assert_eq!(format!("{}", c), "{9 12 15, 19 26 33, 29 40 51}");
        Ok(())
    }

    #[test]
    fn test_matrix_multiplication_error() -> Result<()> {
        let a = Matrix::new(3, 2, [1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(3, 2, [1, 2, 3, 4, 5, 6]);
        let c = multiply_matrix(&a, &b);
        assert!(c.is_err());
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_matrix_multiplication_error2() {
        let a = Matrix::new(3, 2, [1, 2, 3, 4, 5, 6]);
        let b = Matrix::new(2, 4, [1, 2, 3, 4, 5, 6, 7, 8]);
        let _c = b * a;
    }
}
