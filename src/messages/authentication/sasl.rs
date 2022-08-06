mod frontend;
pub use frontend::{SASLInitialResponse, SASLResponse};
mod backend;
use crate::client::{Connection, ConnectionError};
use crate::messages::BackendMessage;
pub use backend::{AuthenticationSASL, AuthenticationSASLContinue, AuthenticationSASLFinal};
use scram::ScramClient;

const SASL_FE_MESSAGE_TYPE: &[u8; 1] = b"p";

// References:
// - https://www.postgresql.org/docs/current/sasl-authentication.html
// - https://github.com/MagicStack/asyncpg/blob/v0.26.0/asyncpg/protocol/scram.pyx
// - https://www.2ndquadrant.com/en/blog/password-authentication-methods-in-postgresql/
// - Relevant RFCs:
//   - RFC 3454
//   - RFC 4013
//   - RFC 4422
//   - RFC 5802
//   - RFC 5803
//   - RFC 7677

pub async fn authenticate(
    connection: &mut Connection,
    user: &str,
    password: &str,
    auth_sasl: AuthenticationSASL,
    // TODO: Replace `ConnectionError` with a proper `SASLAuthError` to be thrown instead
    // of `unwrap`-ing
) -> Result<(), ConnectionError> {
    // TODO: Only SCRAM-SHA-256 is supported, add check here
    let mechanism = auth_sasl.mechanisms[0].to_owned();
    println!("Starting SASL/{} auth...", mechanism);
    let scram = ScramClient::new(user, password, None);
    let (scram, client_first) = scram.client_first();
    let sasl_init_resp = SASLInitialResponse::new(mechanism, client_first);
    connection.write_message(sasl_init_resp).await?;

    match connection.read_message().await? {
        BackendMessage::AuthenticationSASLContinue(sasl_cont) => {
            let scram = scram.handle_server_first(&sasl_cont.server_first).unwrap();
            let (scram, client_final) = scram.client_final();
            let sasl_resp = SASLResponse::new(client_final);
            connection.write_message(sasl_resp).await?;
            match connection.read_message().await? {
                BackendMessage::AuthenticationSASLFinal(sasl_final) => {
                    scram.handle_server_final(&sasl_final.server_final).unwrap();
                    match connection.read_message().await? {
                        BackendMessage::AuthenticationOk(_) => {
                            println!("Auth successfull!");
                            Ok(())
                        }
                        _ => todo!("Error"),
                    }
                }
                _ => todo!("Error"),
            }
        }
        _ => todo!("Error"),
    }
}
