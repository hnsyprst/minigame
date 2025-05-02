pub trait Vector {
    type Scalar;

    fn abs(&self) -> Self::Scalar;
    fn normalize(&self) -> Self;
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
        
        fn abs(&self) -> f32 {
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