use std::{io, thread};
use std::fmt::{Debug, Display, format, Formatter};
use std::future::Future;
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread::JoinHandle;

pub type Result<T> = std::result::Result<T, HttpServerError>;

#[derive(Debug)]
pub enum HttpServerError {
    IOError(io::Error),
    URLError(httparse::Error),
    HandlerError(String),
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

pub async fn start<F, T>(
    addr: impl ToSocketAddrs,
    handler: F,
) -> Result<()>
where
    F: Fn(String, &mut TcpStream) -> T,
    T: Future<Output=std::result::Result<String, String>> + Send + Sync + 'static,
{
    let listener = TcpListener::bind(addr)?;

    // Ok(thread::spawn(move || async {
    for req in listener.incoming() {
        match req {
            Ok(req) => {
                if handle_conn(
                    req,
                    &handler,
                ).await.expect("Error handling connection") {
                    break;
                }
            }
            Err(err) => {
                log::error!("Error reading connect for OAuth server: {}", err.to_string());
            }
        }
    }
    // }))
    Ok(())
}

// Returns if should shut down the server
async fn handle_conn<F, T>(
    mut stream: TcpStream,
    handler: F,
) -> Result<bool>
where
    F: Fn(String, &mut TcpStream) -> T,
    T: Future<Output=std::result::Result<String, String>> + Send + Sync + 'static,
{
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

    let response = handler(path.to_string(), &mut stream).await.map_err(|e| {
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

    #[tokio::test]
    async fn test_server_run() {
        start(SocketAddr::from(([127, 0, 0, 1], 8080)), |path, stream| async {
            Ok(("Hey, you get this?".to_string()))
        }).await.unwrap();
    }
}