use std::future::Future;


pub async fn map_async<T, E, O, Fut, Fun>(
    result: Result<T, E>,
    func: Fun,
) -> Result<O, E>
where
    Fut: Future<Output = O>,
    Fun: FnOnce(T) -> Fut,
{
    match result {
        Ok(val) => { Ok(func(val).await) }
        Err(err) => { Err(err) }
    }
}


pub trait Compress {
    type Type;

    fn compress(self) -> Self::Type;
}

impl<T, E> Compress for Result<Result<T, E>, E> {
    type Type = Result<T, E>;

    fn compress(self) -> Self::Type {
        match self {
            Ok(r) => {
                r
            }
            Err(e) => { Err(e) }
        }
    }
}