#[derive(Debug)]
pub enum LinalgError {
    /// Data size did not match shape.
    SizeMismatch
}

pub trait Vector {
    type Scalar;

    fn abs(&self) -> Self::Scalar;
    fn normalize(&self) -> Self;
    fn dot(&self, rhs: Self) -> Self::Scalar;
    fn angle_to(&self, rhs: Self) -> Self::Scalar;
}

pub mod f32 {
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct Vec2 {
        pub x: f32,
        pub y: f32,
    }

    impl crate::linalg::Vector for Vec2 {
        type Scalar = f32;
        
        fn abs(&self) -> Self::Scalar {
            (self.x.powf(2.0) + self.y.powf(2.0)).sqrt()
        }

        fn normalize(&self) -> Self {
            let mag = self.abs();

            if mag == 0.0 {
                return Vec2 { x: 0.0, y: 0.0 };
            }

            Vec2 {
                x: self.x / mag,
                y: self.y / mag,
            }
        }

        fn dot(&self, rhs: Self) -> Self::Scalar {
            self.x * rhs.x + self.y * rhs.y
        }

        fn angle_to(
            &self,
            rhs: Vec2,
        ) -> Self::Scalar {
            (self.dot(rhs) / (self.abs() * rhs.abs())).acos()
        }
    }

    /// Out of place element-wise vector addition
    impl std::ops::Add<Vec2> for Vec2 {
        type Output = Vec2;

        fn add(self, rhs: Vec2) -> Vec2 {
            Vec2 { 
                x: self.x + rhs.x,
                y: self.y + rhs.y,
            }
        }
    }

    /// In place element-wise vector addition
    impl std::ops::AddAssign<Vec2> for Vec2 {
        fn add_assign(&mut self, rhs: Vec2) {
            self.x += rhs.x;
            self.y += rhs.y;
        }
    }

    /// Out of place element-wise vector subtraction
    impl std::ops::Sub<Vec2> for Vec2 {
        type Output = Vec2;

        fn sub(self, rhs: Vec2) -> Vec2 {
            Vec2 { 
                x: self.x - rhs.x,
                y: self.y - rhs.y,
            }
        }
    }

    /// In place element-wise vector subtraction
    impl std::ops::SubAssign<Vec2> for Vec2 {
        fn sub_assign(&mut self, rhs: Vec2) {
            self.x -= rhs.x;
            self.y -= rhs.y;
        }
    }

    /// Out of place element-wise vector multiplication
    impl std::ops::Mul<Vec2> for Vec2 {
        type Output = Vec2;

        fn mul(self, rhs: Vec2) -> Vec2 {
            Vec2 { 
                x: self.x * rhs.x,
                y: self.y * rhs.y,
            }
        }
    }

    /// In place element-wise vector multiplication
    impl std::ops::MulAssign<Vec2> for Vec2 {
        fn mul_assign(&mut self, rhs: Vec2) {
            self.x *= rhs.x;
            self.y *= rhs.y;
        }
    }

    /// Out of place element-wise vector division
    impl std::ops::Div<Vec2> for Vec2 {
        type Output = Vec2;

        fn div(self, rhs: Vec2) -> Vec2 {
            Vec2 { 
                x: self.x / rhs.x,
                y: self.y / rhs.y,
            }
        }
    }

    /// In place element-wise vector division
    impl std::ops::DivAssign<Vec2> for Vec2 {
        fn div_assign(&mut self, rhs: Vec2) {
            self.x /= rhs.x;
            self.y /= rhs.y;
        }
    }

    /// Out of place Scalar multiplication
    impl std::ops::Mul<f32> for Vec2 {
        type Output = Vec2;

        fn mul(self, rhs: f32) -> Vec2 {
            Vec2 { 
                x: self.x * rhs,
                y: self.y * rhs,
            }
        }
    }

    /// In place Scalar multiplication
    impl std::ops::MulAssign<f32> for Vec2 {
        fn mul_assign(&mut self, rhs: f32) {
            self.x *= rhs;
            self.y *= rhs;
        }
    }
}

pub mod u32 {
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct Vec2 {
        pub x: u32,
        pub y: u32,
    }
}

pub mod u8 {
    use super::LinalgError;

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct Vec2 {
        pub x: u8,
        pub y: u8,
    }

    #[derive(Clone, Debug)]
    pub struct Matrix {
        width: usize,
        height: usize,
        data: Vec<u8>,
    }

    impl Matrix {
        pub fn new(
            width: usize,
            height: usize,
            default_value: u8,
        ) -> Self {
            Matrix { 
                width,
                height,
                data: vec![default_value; width * height],
            }
        }

        pub fn from_vec(
            width: usize,
            height: usize,
            data: Vec<u8>,
        ) -> Result<Self, LinalgError> {
            if data.len() != width * height {
                return Err(LinalgError::SizeMismatch);
            }
            Ok(Matrix {
                width,
                height,
                data,
            })
        }

        pub fn width(&self) -> usize {
            self.width
        }

        pub fn height(&self) -> usize {
            self.height
        }

        pub fn size(&self) -> usize {
            self.data.len()
        }

        fn is_valid_index(
            &self,
            x: usize,
            y: usize,
        ) -> bool {
            x < self.width && y < self.height
        }

        pub fn get(
            &self,
            x: usize,
            y: usize,
        ) -> Option<&u8> {
            if !self.is_valid_index(x, y) { return None }
            Some(&self.data[y * self.width + x])
        }

        pub fn get_mut(
            &mut self,
            x: usize,
            y: usize,
        ) -> Option<&mut u8> {
            if !self.is_valid_index(x, y) { return None }
            Some(&mut self.data[y * self.width + x])
        }

        pub fn get_row(
            &self,
            y: usize,
        ) -> Option<&[u8]> {
            if y > self.height { return None }
            let row_start_idx = y * self.width;
            Some(&self.data[row_start_idx..row_start_idx + self.width])
        }

        // TODO: Maybe return Result?
        pub fn set(
            &mut self,
            x: usize,
            y: usize,
            value: u8,
        ) {
            if let Some(cell) = self.get_mut(x, y) {
                *cell = value
            }
        }

        pub fn iter(&self) -> impl Iterator<Item = &u8> {
            self.data.iter()
        }

        pub fn iter_rows(&self) -> impl Iterator<Item = &[u8]>{
            (0..self.height)
                .map(| row_idx | {
                    self.get_row(row_idx).unwrap()
                })
        }
    }
}