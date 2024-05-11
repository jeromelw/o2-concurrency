// Code for matrix multiplication
use anyhow::Result;
use concurrency::Matrix;

fn main() -> Result<()> {
    let a = Matrix::new(3, 2, [1, 2, 3, 4, 5, 6]);
    let b = Matrix::new(2, 4, [1, 2, 3, 4, 5, 6, 7, 8]);
    let c = a * b;
    println!("{}", c);

    Ok(())
}
