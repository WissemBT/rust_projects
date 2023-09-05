use super::gamma::gamma_correct;
use core::mem::transmute;
use micromath::F32Ext;

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn gamma_correct(&self) -> Self {
        Color {
            r: gamma_correct(self.r),
            g: gamma_correct(self.g),
            b: gamma_correct(self.b),
        }
    }
}
pub const RED: Color = Color { r: 255, g: 0, b: 0 };
pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

impl core::ops::Mul<f32> for Color {
    type Output = Color;
    fn mul(self, rhs: f32) -> Self::Output {
        let red = (self.r as f32 * rhs).max(0.0).min(255.0).round();
        let green = (self.g as f32 * rhs).max(0.0).min(255.0).round();
        let blue = (self.b as f32 * rhs).max(0.0).min(255.0).round();
        Color {
            r: red as u8,
            g: green as u8,
            b: blue as u8,
        }
    }
}
impl core::ops::Div<f32> for Color {
    type Output = Color;
    fn div(self, rhs: f32) -> Self::Output {
        self * (1_f32 / rhs)
    }
}

#[repr(transparent)]
pub struct Image([Color; 64]);

impl Image {
    pub fn new_solid(color: Color) -> Self {
        Image([color; 64])
    }

    pub fn row(&self, row: usize) -> &[Color] {
        &self.0[8 * row..8 * (row + 1)]
    }

    pub fn gradient(color: Color) -> Self {
        let mut result = Image::default();
        for row in 0..8 {
            for col in 0..8 {
                let div = 1 + row * row + col;
                result[(row, col)] = color / div as f32;
            }
        }
        result
    }
}

impl Default for Image {
    fn default() -> Self {
        Image([Color::default(); 64])
    }
}

impl core::ops::Index<(usize, usize)> for Image {
    type Output = Color;
    fn index(&self, idx: (usize, usize)) -> &Self::Output {
        &self.0[8 * idx.0 + idx.1]
    }
}

impl core::ops::IndexMut<(usize, usize)> for Image {
    fn index_mut(&mut self, idx: (usize, usize)) -> &mut Self::Output {
        &mut self.0[8 * idx.0 + idx.1]
    }
}

impl AsRef<[u8; 192]> for Image {
    fn as_ref(&self) -> &[u8; 192] {
        unsafe { transmute(self) }
    }
}

impl AsMut<[u8; 192]> for Image {
    fn as_mut(&mut self) -> &mut [u8; 192] {
        unsafe { transmute(self) }
    }
}
