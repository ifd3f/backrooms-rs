use ndarray::Array2;

use crate::camera::RaycastableWorld;

#[derive(Debug, Clone)]
pub struct ArrayWorld {
    map: Array2<bool>,
}

impl RaycastableWorld for ArrayWorld {
    fn exists(&self, (x, y): (isize, isize)) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        self.map
            .get((y as usize, x as usize))
            .copied()
            .unwrap_or(false)
    }
}

impl From<Array2<bool>> for ArrayWorld {
    fn from(map: Array2<bool>) -> Self {
        Self { map }
    }
}
