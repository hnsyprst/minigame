use std::{cell::RefCell, rc::Rc};

use miniquad::fs::load_file;

use crate::linalg::{u32, u8};

type ResourceId = u16;

#[derive(Debug)]
pub enum ResourceError {
    /// A Resource ID was out of bounds.
    OutOfBounds,
    /// Failed to parse a resource.
    ParseError,
    /// Miniquad failed to load a file.
    MiniquadFsError(miniquad::fs::Error),
    /// Tried to parse a resource before it was loaded.
    ResourceNotReady,
}

#[derive(Debug)]
pub struct Resource {
    pub id: ResourceId,
}

pub struct ResourceManager {
    resource_bytes: Vec<Option<Vec<u8>>>,
    resources_to_load: Vec<String>,
}

impl ResourceManager {
    pub fn new() -> ResourceManager {
        ResourceManager {
            resource_bytes: Vec::new(),
            resources_to_load: Vec::new(),
        }
    }

    pub fn register_resource(
        &mut self,
        resource_path: &str,
    ) -> Resource {
        self.resource_bytes.push(None);
        self.resources_to_load.push(resource_path.to_string());
        Resource {
            id: self.resource_bytes.len() as ResourceId - 1,
        }
    }

    pub fn get_as_bytes(
        &self,
        resource: &Resource,
    ) -> Result<&Vec<u8>, ResourceError> {
        match self.resource_bytes.get(resource.id as usize).ok_or(ResourceError::OutOfBounds)? {
            Some(bytes) => Ok(bytes),
            None => Err(ResourceError::ResourceNotReady),
        }
    }

    fn get_as_bytes_mut(
        &mut self,
        resource: &Resource,
    ) -> Result<&mut Vec<u8>, ResourceError> {
        match self.resource_bytes.get_mut(resource.id as usize).ok_or(ResourceError::OutOfBounds)? {
            Some(bytes) => Ok(bytes),
            None => Err(ResourceError::ResourceNotReady),
        }
    }

    pub fn get_as_rgba8(
        &self,
        resource: &Resource,
        texture_size: &u32::Vec2,
    ) -> Result<Vec<u8>, ResourceError> {
        let bytes = self.get_as_bytes(resource)?;
        // Decode png as rgba8
        let (_, mut pixels) = png_decoder::decode(bytes).map_err(| _err | {ResourceError::ParseError})?;

        // png-decoder decodes the image flipped, so flip it
        let row_length = texture_size.x as usize * 4; // 4 u8s per pixel (rgba8)
        for y in 0..texture_size.y as usize / 2 {
            let top_index = y * row_length;
            let bottom_index = (texture_size.y as usize - 1 - y) * row_length;

            let (top_slice, bottom_slice) = pixels.split_at_mut(bottom_index);
            top_slice[top_index..top_index + row_length].swap_with_slice(
                &mut bottom_slice[..row_length],
            );
        }
        Ok(pixels)
    }

    pub fn load_resources(&mut self) -> Result<(), ResourceError> {
        let mut pending_count: usize = 0;
        let loaded_bytes = Rc::new(RefCell::new(Vec::new()));

        for ((id, resource_path), resource_bytes) in self.resources_to_load.iter().enumerate().zip(self.resource_bytes.iter_mut()) {
            // Already loaded, skip this file
            if resource_bytes.is_some() {
                continue;
            }
            pending_count += 1;
            let loaded_bytes_ref = loaded_bytes.clone();
            load_file(resource_path, move | bytes | {
                loaded_bytes_ref
                    .borrow_mut()
                    .push(( Resource { id: id as ResourceId }, bytes ));
            });
        }

        // Block until all pending files have been loaded
        while pending_count != loaded_bytes.borrow().len() { };

        for (resource, maybe_bytes) in loaded_bytes.borrow_mut().drain(..) {
            let bytes = maybe_bytes.map_err(| err | { ResourceError::MiniquadFsError(err) })?;
            self.resource_bytes[resource.id as usize] = Some(bytes);
        };
        Ok(())
    }
}