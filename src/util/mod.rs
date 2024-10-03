#[macro_export]
macro_rules! try_do {
    (result $expr: expr) => {
        match $expr {
            Ok(value) => value,
            Err(_) => return None
        }
    };
    ($expr: expr) => {
        match $expr {
            Some(value) => value,
            None => return None
        }
    };
    ($expr: expr; or continue) => {
        match $expr {
            Some(value) => value,
            None => continue
        }
    };
    ($expr: expr; or break) => {
        match $expr {
            Some(value) => value,
            None => break
        }
    };
    ($expr: expr; or return $err_value: expr) => {
        match $expr {
            Some(value) => value,
            None => return $err_value
        }
    };
}
