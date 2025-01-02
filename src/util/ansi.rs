#![allow(unused)]

use std::io::IsTerminal;

static mut IS_ANSI_ENABLED: bool = false;

pub fn enable_ansi() {
    if !std::io::stdout().is_terminal() {
        unsafe {
            IS_ANSI_ENABLED = false;
        }
        return;
    }

    let success = match enable_ansi_support::enable_ansi_support() {
        Ok(()) => true,
        Err(_) => false,
    };

    unsafe {
        IS_ANSI_ENABLED = success;
    }
}

pub fn disable_ansi() {
    unsafe { IS_ANSI_ENABLED = false }
}

pub fn is_ansi_enabled() -> bool {
    unsafe { IS_ANSI_ENABLED }
}

pub mod colors {
    use crate::util::ansi::is_ansi_enabled;

    pub static BLACK: &str = "\x1b[30m";
    pub static RED: &str = "\x1b[31m";
    pub static GREEN: &str = "\x1b[32m";
    pub static YELLOW: &str = "\x1b[33m";
    pub static BLUE: &str = "\x1b[34m";
    pub static PURPLE: &str = "\x1b[35m";
    pub static CYAN: &str = "\x1b[36m";
    pub static WHITE: &str = "\x1b[37m";
    pub static DEFAULT: &str = "\x1b[39m";
    pub static RESET: &str = "\x1b[0m";
    static EMPTY_STRING: &str = "";

    macro_rules! create_fn {
        ($fn_name: ident, $str_name: ident) => {
            pub fn $fn_name() -> &'static str {
                if is_ansi_enabled() {
                    $str_name
                } else {
                    EMPTY_STRING
                }
            }
        };
    }

    create_fn!(black, BLACK);
    create_fn!(red, RED);
    create_fn!(green, GREEN);
    create_fn!(yellow, YELLOW);
    create_fn!(blue, BLUE);
    create_fn!(purple, PURPLE);
    create_fn!(cyan, CYAN);
    create_fn!(white, WHITE);
    create_fn!(default, DEFAULT);
    create_fn!(reset, RESET);
}
