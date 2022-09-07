use regex::Regex;
pub mod error;
pub use error::InvalidDsnError;

const PATTERN: &str = r"^(?P<driver>postgres(ql)?)://(((?P<user>[\w\d]+)?(:(?P<password>[^@/:\s]+))?@)?(?P<netloc>[\w\d]+)(:(?P<port>\d+))?/?(?P<dbname>[\w\d]+)?(\?)?(?P<params>.*))?$";

#[derive(Debug)]
pub struct Dsn {
    pub user: String,
    pub address: String,
    pub password: Option<String>,
    pub dbname: Option<String>,
    pub params: Option<String>,
}

pub fn parse_dsn(dsn: &str) -> Result<Dsn, InvalidDsnError> {
    let re = Regex::new(PATTERN).unwrap();
    let caps = re.captures(dsn).unwrap();
    match (caps.name("user"), caps.name("netloc")) {
        (Some(user_match), Some(netloc_match)) => {
            let user = user_match.as_str().to_owned();
            let netloc = netloc_match.as_str();
            let address = caps.name("port").map_or(netloc.to_owned(), |port| {
                format!("{}:{}", netloc, port.as_str())
            });
            let password = caps.name("password").map(|v| v.as_str().to_owned());
            let dbname = caps.name("dbname").map(|v| v.as_str().to_owned());
            let params = caps.name("params").map(|v| v.as_str().to_owned());
            let dsn = Dsn {
                user,
                address,
                password,
                dbname,
                params,
            };
            Ok(dsn)
        }
        (Some(_), None) => Err(InvalidDsnError::MissingUser),
        (None, Some(_)) => Err(InvalidDsnError::MissingNetloc),
        (None, None) => Err(InvalidDsnError::MissingUserAndNetloc),
    }
}
