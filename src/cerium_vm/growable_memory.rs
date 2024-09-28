pub struct GrowableMemoryBlock {
    memory: Vec<u8>,
}

impl GrowableMemoryBlock {
    const INITIAL_MEMORY: usize = 1 << 8;
    const MAX_MEMORY: usize = 1 << 16;

    pub fn new() -> Self {
        GrowableMemoryBlock { memory: Vec::from([0; Self::INITIAL_MEMORY]) }
    }

    pub fn resize_to_fit(&mut self, ptr: usize) -> Result<(), String> {
        if ptr > Self::MAX_MEMORY {
            Err(format!(
                "CeriumVM error: memory size cannot exceed {} bytes",
                Self::MAX_MEMORY
            ).to_owned())
        } else {
            if ptr > self.memory.len() {
                self.memory.resize(usize::next_power_of_two(ptr), 0);
            }

            Ok(())
        }
    }

    pub fn at<T>(&mut self, ptr: usize) -> Result<&mut T, String> {
        match self.resize_to_fit(ptr + size_of::<T>()) {
            Ok(_) => unsafe {
                Ok(self.memory.as_mut_ptr().add(ptr).cast::<T>().as_mut().unwrap())
            }
            Err(err) => {
                Err(err)
            }
        }
    }
}