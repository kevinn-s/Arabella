use alloc::vec::Vec;
use bytemuck::{Pod, Zeroable, cast};
use fearless_simd::*;
use lyon_geom::{Point, Box2D, Transform};
use lyon_path::{Event, Iter};

use crate::{TILE_HEIGHT, TILE_WIDTH};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Tile {
    pub x: u16,
    pub y: u16,
    pub width: u8,
    pub height: u8,
    pub _pad: [u8; 2],
    pub backdrop: [i16; 8],
    pub segments: [f32; 2],
    pub payload: u32,
    pub paint_and_rect_flag: u32,
    pub depth_index: u32,
}

#[derive(Clone, Debug)]
pub struct TileMap<T>
where
    T: Clone + Copy,
{
    pub data: Vec<T>,
}

impl<T> TileMap<T>
where
    T: Clone + Copy,
{
    #[inline]
    pub fn new<F>(mut tile: F) -> TileMap<T>
    where
        F: FnMut() -> T,
    {
        TileMap { data: Vec::new() }
    }

    #[inline]
    pub fn push(&mut self, tile: T) {
        self.data.push(tile);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    pub fn get(&mut self, coords: Point<f32>, bounds: Box2D<f32>) -> Option<&mut T> {
        if bounds.contains(coords) {
            let width = bounds.width() as usize;
            let x = (coords.x - bounds.min.x) as usize;
            let y = (coords.y - bounds.min.y) as usize;
            self.data.get_mut(y * width + x)
        } else {
            None
        }
    }

    #[inline]
    pub fn coordinate_to_index(&self, coords: Point<f32>, bounds: Box2D<f32>) -> usize {
        let width = bounds.width() as usize;
        let x = (coords.x - bounds.min.x) as usize;
        let y = (coords.y - bounds.min.y) as usize;
        y * width + x
    }
}

