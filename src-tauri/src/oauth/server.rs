use std::{io, thread};
use std::fmt::{Debug, Display, format, Formatter};
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread::JoinHandle;

pub type Result<T> = std::result::Result<T, HttpServerError>;

#[derive(Debug)]
pub enum HttpServerError {
    IOError(io::Error),
    URLError(httparse::Error),
    HandlerError(String)
}

impl Display for HttpServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            HttpServerError::IOError(e) => {
                format!(
                    "IOError: {}",
                    e.to_string()
                )
            }
            HttpServerError::URLError(e) => {
                format!(
                    "URL Error: {}",
                    e.to_string()
                )
            }
            HttpServerError::HandlerError(e) => {
                format!(
                    "Internal handler error: {}",
                    e
                )
            }
        };
        write!(f, "{}", str)
    }
}

impl From<io::Error> for HttpServerError {
    fn from(value: Error) -> Self {
        HttpServerError::IOError(value)
    }
}

pub fn start(
    addr: impl ToSocketAddrs,
    handler: impl Fn(&str, &mut TcpStream) -> std::result::Result<String, String> + Send + Sync + 'static
) -> Result<JoinHandle<()>> {
    let listener = TcpListener::bind(addr)?;

   Ok(thread::spawn(move || {
        for req in listener.incoming() {
            match req {
                Ok(req) => {
                    if handle_conn(
                        req,
                        &handler
                    ).expect("Error handling connection") {
                        break
                    }
                }
                Err(err) => {
                    log::error!("Error reading connect for OAuth server: {}", err.to_string());
                }
            }
        }
    }))
}

// Returns if should shut down the server
fn handle_conn(
    mut stream: TcpStream,
    handler: &impl Fn(&str, &mut TcpStream) -> std::result::Result<String, String>
) -> Result<bool> {
    let mut buffer = [0; 4048];
    if let Err(io_err) = stream.read(&mut buffer) {
        log::error!("Error reading connect headers: {}", io_err.to_string());
    };
    if buffer[..4] == [1, 3, 3, 7] {
        return Ok(false);
    }

    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut request = httparse::Request::new(&mut headers);
    request.parse(&buffer).map_err(|e| {
        HttpServerError::URLError(e)
    })?;

    let path = request.path.unwrap_or_default();

    let response = handler(path, &mut stream).map_err(|e| {
        HttpServerError::HandlerError(e)
    });

    let string = response.unwrap_or_else(|err| err.to_string());
    stream.write_all(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            string.len(),
            string
        ).as_bytes(),
    )?;

    stream.flush()?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use crate::oauth::server::start;

    #[test]
    fn test_server_run() {
        start(SocketAddr::from(([127, 0, 0, 1], 8080)), |path, stream| {
            Ok(("Hey, you get this?".to_string()))
        }).unwrap().join().unwrap();
    }
}