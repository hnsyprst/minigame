use std::{cell::RefCell, collections::HashMap, rc::Rc};

use miniquad::fs::load_file;

type ResourceId = u16;

#[derive(Debug)]
pub enum ResourceError {
    /// A Resource ID was out of bounds
    OutOfBounds,
    MiniquadFsError,
}

#[derive(Debug)]
pub struct Resource {
    pub id: ResourceId,
}

/// 
pub struct ResourceManager {
    resource_bytes: Vec<Option<Vec<u8>>>,
    resources_to_load: Vec<String>,
}


// FIXME: Always treating resources as Option is kind of silly
// it would be better to just return Err if the resource isn't loaded;
// its assumed that you'll call `load_resources()` before you start 
// trying to interact with your resources
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
    ) -> Result<&Option<Vec<u8>>, ResourceError> {
        match self.resource_bytes.get(resource.id as usize) {
            Some(maybe_resource_bytes) => return Ok(maybe_resource_bytes),
            None => return Err(ResourceError::OutOfBounds),
        }
    }

    fn get_as_bytes_mut(
        &mut self,
        resource: &Resource,
    ) -> Result<&mut Option<Vec<u8>>, ResourceError> {
        match self.resource_bytes.get_mut(resource.id as usize) {
            Some(maybe_resource_bytes) => return Ok(maybe_resource_bytes),
            None => return Err(ResourceError::OutOfBounds),
        }
    }

    pub fn get_as_rgba8(
        &self,
        resource: &Resource,
    ) -> Result<Option<Vec<u8>>, ResourceError> {
        match self.get_as_bytes(resource)? {
            Some(bytes) => {
                let (_, pixels) = png_decoder::decode(bytes).unwrap();
                Ok(Some(pixels))
            }
            None => return Ok(None),
        }
    }

    pub fn load_resources(&mut self) {
        let pending = Rc::new(RefCell::new(Vec::new()));

        for ((id, resource_path), resource_bytes) in self.resources_to_load.iter().enumerate().zip(self.resource_bytes.iter_mut()) {
            if resource_bytes.is_some() {
                continue;
            }
            let this_pending = pending.clone();
            load_file(resource_path, move | bytes | {
                this_pending.borrow_mut().push(( Resource { id: id as ResourceId }, bytes ));
            });
        }
        
        while !self.resource_bytes.iter().all(| bytes | bytes.is_some()) {
            for (resource, maybe_bytes) in pending.borrow_mut().drain(..) {
                // TODO: For now, we're just going to panic on any file load errors
                let resource_bytes = self.get_as_bytes_mut(&resource).expect("Internal resource error");
                if resource_bytes.is_some() { panic!("Tried to load into an already loaded resource!") };
                let bytes = maybe_bytes.expect("File load error!");
                *resource_bytes = Some(bytes);
            }
        }

    }
}