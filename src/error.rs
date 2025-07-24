#![allow(non_local_definitions, dead_code)]
use {
    diesel::r2d2,
    std::{env::VarError, io},
    tokio::task::JoinError,
};

#[macro_export]
macro_rules! error_custom {
    ($msg:literal) => {
        Error::CustomError(500, $msg.to_string())
    };
}

/// List of possible API errors.
#[derive(Fail, Debug)]
pub enum Error {
    /// Storage error. This type includes errors related to the database, caused
    /// by, for example, serialization issues.
    #[fail(display = "Database error: {}", _0)]
    Database(#[cause] diesel::result::Error),

    /// Input/output error. This type includes errors related to files that are
    /// not a part of the Exonum storage.
    #[fail(display = "IO error: {}", _0)]
    Io(#[cause] io::Error),

    /// Bad request. This error occurs when the request contains invalid syntax.
    #[fail(display = "Bad request: {}", _1)]
    BadRequest(i32, String),

    /// Not found. This error occurs when the server cannot locate the requested
    /// resource.
    #[fail(display = "Not found: {}", _1)]
    NotFound(i32, String),

    /// Internal server error. This type can return any internal server error to
    /// the user.
    #[fail(display = "Internal server error: {}", _1)]
    InternalError(i32, #[cause] failure::Error),

    /// Error yang muncul apabila user menginputkan parameter yang tidak sesuai
    #[fail(display = "Invalid parameter: {}", _1)]
    InvalidParameter(i32, String),

    /// Error yang muncul ketika sebuah object unik telah ada
    /// biasanya dimunculkan oleh operasi creation.
    #[fail(display = "Already exists")]
    AlreadyExists,

    /// Error yang muncul ketika suatu object telah habis masa berlakunya
    /// pada saat transaksi misalnya.
    #[fail(display = "{} expired", _0)]
    Expired(&'static str),

    /// Error yang bisa digunakan untuk menampilkan kode dan deskripsi secara
    /// custom.
    #[fail(display = "{}", _0)]
    CustomError(i32, String),

    /// Deserialize error. This error occurs when the request contains invalid
    /// syntax.
    #[fail(display = "Deserialize error: {}", _0)]
    Deserialize(#[cause] serde_json::Error),

    /// Unauthorized error. This error occurs when the request lacks valid
    /// authentication credentials.
    #[fail(display = "Unauthorized: {}", _0)]
    Unauthorized(String),

    /// Fireblocks error
    #[fail(display = "Fireblocks error: {}", _0)]
    FireblocksError(String),

    /// Forbidden error. This error occurs when the server refuses to authorize
    #[fail(display = "Forbidden: {}", _0)]
    Forbidden(i32, String),
}

/// Definisi kode kesalahan
pub enum ErrorCode {
    /// Sukses atau tidak terjadi error.
    NoError = 0,
    /// Unauthorized
    Unauthorized = 3000,
    /// Forbidden
    Forbidden = 3001,
    /// User restricted
    RestrictedUser = 3002,

    /// Kegagalan yang berkaitan dengan proses serialize/deserialize data.
    SerializeDeserializeError = 4001,
    /// Parameter tidak lengkap/kurang.
    InvalidParameter = 4002,
    /// Message tidak ada signature-nya, dibutuhkan untuk verifikasi menggunakan
    /// public key.
    MessageHasNoSign = 4003,
    /// Tidak ada informasi login.
    NoLoginInfo = 4004,
    /// Pengirim dan penerima alamatnya sama.
    FromAndToTargetIsSame = 4005,

    /// Kegagalan yang tidak diketahui penyebabnya.
    UnknownError = 5001,

    /// Kegagalan pada database internal apabila terjadi error.
    DatabaseError = 6001,
    /// Kegagalan pada database yang berkaitan dengan
    /// ketidakditemukannya record/data di dalam database.
    DatabaseRecordNotFoundError = 6002,
    // Tambahkan definisi kode error mu sendiri di sini.
}

impl Error {
    /// Convert a `diesel::result::Error` into a `Error` instance.
    ///
    /// If the error is a `diesel::result::Error::NotFound`, it will be
    /// converted to a `BadRequest` error with the code
    /// `ErrorCode::DatabaseRecordNotFoundError`. Otherwise, it will be
    /// converted to a `Database` error with the original error.
    ///
    /// # Arguments
    ///
    /// * `err`: The `diesel::result::Error` to convert.
    /// * `name`: The name of the record that was not found. This is used to
    ///   construct the error message.
    ///
    /// # Returns
    ///
    /// A `Error` instance.
    pub fn from_diesel(err: diesel::result::Error, message: String) -> Error {
        use diesel::result::Error as DbError;

        match err {
            DbError::NotFound => {
                Self::BadRequest(ErrorCode::DatabaseRecordNotFoundError as i32, message)
            }
            _ => Self::Database(err),
        }
    }
}

impl From<VarError> for Error {
    fn from(value: VarError) -> Self {
        match value {
            VarError::NotPresent => Error::InvalidParameter(
                ErrorCode::InvalidParameter as i32,
                "Environment variable not found".to_string(),
            ),
            VarError::NotUnicode(os_string) => Error::InvalidParameter(
                ErrorCode::InvalidParameter as i32,
                format!(
                    "Environment variable is not unicode: {}",
                    os_string.to_string_lossy()
                ),
            ),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Deserialize(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::InternalError(ErrorCode::UnknownError as i32, value.into())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::InternalError(ErrorCode::UnknownError as i32, value.into())
    }
}

impl From<diesel::result::Error> for Error {
    fn from(value: diesel::result::Error) -> Self {
        Error::InternalError(ErrorCode::DatabaseError as i32, value.into())
    }
}

impl From<JoinError> for Error {
    fn from(value: JoinError) -> Self {
        Error::InternalError(ErrorCode::UnknownError as i32, value.into())
    }
}

impl From<r2d2::Error> for Error {
    fn from(value: r2d2::Error) -> Self {
        Error::InternalError(ErrorCode::UnknownError as i32, value.into())
    }
}
// use scraper::error::SelectorErrorKind;

// impl From<SelectorErrorKind<'_>> for Error {
//     fn from(value: SelectorErrorKind) -> Self {
//         match value {
//             SelectorErrorKind::UnexpectedToken(_) => error_custom!("Unexpected token"),
//             SelectorErrorKind::EndOfLine => error_custom!("End of line"),
//             SelectorErrorKind::InvalidAtRule(_) => error_custom!("Invalid at rule"),
//             SelectorErrorKind::InvalidAtRuleBody => error_custom!("Invalid at rule body"),
//             SelectorErrorKind::QualRuleInvalid => error_custom!("Qual rule invalid"),
//             SelectorErrorKind::ExpectedColonOnPseudoElement(_) => {
//                 error_custom!("Expected colon on pseudo element")
//             }
//             SelectorErrorKind::ExpectedIdentityOnPseudoElement(_) => {
//                 error_custom!("Expected identity on pseudo element")
//             }
//             SelectorErrorKind::UnexpectedSelectorParseError(_) => {
//                 error_custom!("Unexpected selector parse error")
//             }
//         }
//     }
// }
