mod frontend;
pub use frontend::{SASLInitialResponse, SASLResponse};
mod backend;
pub use backend::{AuthenticationSASL, AuthenticationSASLContinue, AuthenticationSASLFinal};

const SASL_FE_MESSAGE_TYPE: &[u8; 1] = b"p";

// References:
// - https://www.postgresql.org/docs/current/sasl-authentication.html
// - https://github.com/MagicStack/asyncpg/blob/075114c195e9eb4e81c8365d81540beefb46065c/asyncpg/protocol/scram.pyx
// - https://www.2ndquadrant.com/en/blog/password-authentication-methods-in-postgresql/
// - Relevant RFCs:
//   - RFC 3454
//   - RFC 4013
//   - RFC 4422
//   - RFC 5802
//   - RFC 5803
//   - RFC 7677
