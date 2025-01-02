use std::fmt::{Display, Formatter};
use std::panic::panic_any;

pub trait CeriumError: Display + Send
where
    Self: 'static,
{
    fn message(&self) -> String;

    fn throw(self) -> ! where Self: Sized {
        panic_any(Box::new(self) as Box<dyn CeriumError>);
    }
}

macro_rules! create_basic_error_type {
    ($name : ident, $format_str: expr) => {
        pub struct $name {
            message: String,
        }
        
        #[allow(unused)]
        impl $name {
            pub fn throw_str(message: &str) -> ! {
                Self { message: message.to_owned() }.throw()
            }

            pub fn throw_string(message: String) -> ! {
                Self { message }.throw()
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.message.fmt(f)
            }
        }

        impl CeriumError for $name {
            fn message(&self) -> String {
                let message = &self.message;
                format!($format_str, message)
            }
        }
    };
}

create_basic_error_type!(CeriumVMError, "CeriumVM Error: {}");
create_basic_error_type!(CeriumAssemblerError, "CeriumAssembler Error: {}");
create_basic_error_type!(CeriumVMHeapAccessError, "CeriumVM Heap Access Error: {}");
create_basic_error_type!(CeriumVMInternalError, "Internal CeriumVM Error: {}");