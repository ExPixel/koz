use std::{env, path::Path, str::FromStr};

use anyhow::Context as _;

pub fn load_dotenv() -> anyhow::Result<Vec<&'static Path>> {
    #[cfg(test)]
    let paths = [
        Path::new(".env.test"),
        Path::new("config/.env.test"),
        Path::new(".env.local"),
        Path::new("config/.env.local"),
        Path::new(".env"),
        Path::new("config/.env"),
    ];

    #[cfg(not(test))]
    let paths = [
        Path::new(".env.local"),
        Path::new("config/.env.local"),
        Path::new(".env"),
        Path::new("config/.env"),
    ];

    let mut loaded = Vec::with_capacity(paths.len());

    for path in paths {
        if self::load_dotenv_path(path)? {
            loaded.push(path);
        }
    }

    Ok(loaded)
}

pub fn load_dotenv_path<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<bool> {
    let path = path.as_ref();
    match dotenvy::from_path(path) {
        Ok(_) => Ok(true),
        Err(dotenvy::Error::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(anyhow::Error::new(err)),
    }
}

pub fn parse_opt_required<T>(name: &str) -> anyhow::Result<T>
where
    T: 'static + FromStr,
    T::Err: std::error::Error + Send + Sync,
{
    let value = parse_opt(name)?
        .ok_or_else(|| anyhow::anyhow!("required environment variable `{name}` not set"))?;
    Ok(value)
}

pub fn parse_opt<T>(name: &str) -> anyhow::Result<Option<T>>
where
    T: 'static + FromStr,
    T::Err: std::error::Error + Send + Sync,
{
    let string_value = match env::var(name) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(env::VarError::NotUnicode(value)) => {
            anyhow::bail!("environment variable value is not unicode: {value:?}")
        }
    };
    let str_value = string_value.as_str();
    let value: T = str_value.parse().with_context(|| {
        let type_name = std::any::type_name::<T>();
        format!("invalid {type_name}: {str_value}")
    })?;

    Ok(Some(value))
}
