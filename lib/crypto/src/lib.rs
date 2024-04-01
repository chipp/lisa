mod token;
pub use token::{parse_token, Token};

pub mod cbc {
    mod dec;
    mod enc;

    pub use dec::decrypt;
    pub use enc::encrypt;
}

pub mod ebc {
    mod dec;

    pub use dec::decrypt;
}
