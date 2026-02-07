use std::error::Error;

pub type AppResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;
