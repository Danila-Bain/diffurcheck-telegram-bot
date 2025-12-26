use nalgebra::*;

const LIMIT: i32 = 2;

fn main() {
    let values = [-1.0, 0.0, 1.0];

    'd: for n in 0..3usize.pow(9) {
        let mut m = n;
        let mut data = [0.0; 9];
        let mut norm = 0.;
        for i in 0..9 {
            data[i] = values[m % 3];
            norm += data[i].abs();
            m /= 3;
        }
        let mat = Matrix3::from_row_slice(&data);
        
        let j = Matrix3::from_row_slice(&[1.,0.,0.,0.,2.,0.,0.,0.,-1.]);

        if norm >= 7. && let Some(inv) = mat.try_inverse() {
            let elements = inv.as_slice();
            for el in elements.iter() {
                if &el.floor() != el {
                    continue 'd;
                }
            }

            let mat_a = mat * j * inv;

            println!("Matrix:{mat}Inverse:{inv}New:{mat_a}\n\n");
        }
    }
}
