use nalgebra::*;

const LIMIT: i32 = 3;

fn main() {
    for a in -LIMIT..=LIMIT {
        for b in -LIMIT..=LIMIT {
            for c in -LIMIT..=LIMIT {
                'd: for d in -LIMIT..=LIMIT {
                    let mat = Matrix2::new(a as f64, b as f64, c as f64, d as f64);

                    if let Some(inv) = mat.try_inverse() {

                        let elements = inv.as_slice();
                        for el in elements.iter() {
                            if &el.floor() != el || el == &0. {
                                continue 'd;
                            }
                        }

                        println!("Matrix:{mat}Inverse:{inv}\n\n");

                    }
                }
            }
        }
    }
}
