pub mod f32 {
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct Vec2 {
        pub x: f32,
        pub y: f32,
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
