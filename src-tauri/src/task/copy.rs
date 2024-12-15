use crate::task::{ProgressTracker, ProgressUpdate, ProgressUpdateAsync};
use futures::{Stream, StreamExt};
use std::fmt::{Debug, Display};
use std::hash::Hasher;
use std::io::{Read, Write};
use tokio::io;

// pub fn copy_tracking<R, W, P: ProgressTracker>(
//     reader: &mut R,
//     writer: &mut W,
//     size: f64, // in bytes
//     tracker: &mut P,
// ) -> io::Result<()> where
//     R: Read,
//     W: Write,
// {
//     let mut bytes_read = 0f64;
//     let mut buf = [0; 1024];
//     loop {
//         let read = reader.read(&mut buf)?;
//         writer.write_all(&buf[..read])?;
//
//         bytes_read = bytes_read + read as f64;
//         tracker.update(bytes_read / size);
//
//         if read < 1024 {
//             return Ok(());
//         }
//     }
// }

pub async fn copy_stream_tracking<I, ReadE, E, R, W>(
    stream: &mut R,
    writer: &mut W,
    size: u64, // in bytes
    tracker: &mut dyn ProgressUpdate,
) -> Result<(), E>
where
    I: Into<Vec<u8>>,
    E: From<io::Error> + From<ReadE>,
    ReadE: Display + 'static,
    R: Stream<Item = Result<I, ReadE>> + Unpin,
    W: Write,
{
    let mut bytes_read = 0u64;
    loop {
        let read = stream.next().await;

        if let Some(result) = read {
            match result {
                Ok(bytes) => {
                    let read = bytes.into();
                    writer.write_all(&read)?;

                    bytes_read = bytes_read + read.len() as u64;
                    tracker.update((bytes_read as f64) / (size as f64));
                }
                Err(e) => {
                    tracker.erroneously_complete(&e);
                    return Err(e.into());
                }
            }
        } else {
            return Ok(());
        }
    }
}

pub async fn copy_stream_tracking_async<I, ReadE, E, R, W, P: ProgressUpdateAsync>(
    stream: &mut R,
    writer: &mut W,
    size: u64,
    tracker: &mut P,
) -> Result<(), E>
where
    I: Into<Vec<u8>>,
    E: From<io::Error> + From<ReadE>,
    ReadE: Display + 'static,
    R: Stream<Item = Result<I, ReadE>> + Unpin,
    W: Write,
{
    let mut bytes_read = 0u64;
    loop {
        let read = stream.next().await;

        if let Some(result) = read {
            match result {
                Ok(bytes) => {
                    let read = bytes.into();
                    writer.write_all(&read)?;

                    bytes_read = bytes_read + read.len() as u64;
                    tracker.update((bytes_read as f64) / (size as f64)).await
                }
                Err(e) => {
                    tracker.erroneously_complete(&e).await;
                    return Err(e.into());
                }
            }
        } else {
            return Ok(());
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::task::copy::{copy_stream_tracking, copy_stream_tracking_async};
    use crate::task::tests::PrintingProgressTracker;
    use std::any::Any;
    use std::fmt::{Debug, Display, Formatter};
    use std::fs::{create_dir_all, File};
    use std::path::PathBuf;

    #[derive(Debug)]
    enum Error {
        Io(std::io::Error),
        Network(reqwest::Error),
    }

    impl Display for Error {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                Error::Io(e) => e.to_string(),
                Error::Network(e) => e.to_string(),
            };

            write!(f, "{}", s)
        }
    }

    impl From<reqwest::Error> for Error {
        fn from(value: reqwest::Error) -> Self {
            Error::Network(value)
        }
    }

    impl From<std::io::Error> for Error {
        fn from(value: std::io::Error) -> Self {
            Error::Io(value)
        }
    }

    #[tokio::test]
    async fn test_tracking_copy() {
        let url = "https://maven.extframework.dev/releases/dev/extframework/client/1.0-BETA/client-1.0-BETA-all.jar";
        let response = reqwest::get(url).await.unwrap();
        let mut stream = response.bytes_stream();

        create_dir_all("tests").unwrap();
        let output_path = PathBuf::from("tests/download.jar");
        let mut file = File::create(output_path).unwrap();

        let mut tracker = PrintingProgressTracker {
            percent: 0.0,
            erroneous: false,
            file: File::create("tests/output.txt").unwrap(),
            name: "test".to_string(),
        };

        let r: Result<(), Error> = copy_stream_tracking(
            &mut stream,
            &mut file,
            23100000u64, // ~23.1 MB
            &mut tracker,
        )
        .await;
        r.unwrap();
    }
}
