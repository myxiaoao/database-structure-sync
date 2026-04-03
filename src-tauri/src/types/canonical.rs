/// Database-agnostic intermediate type for cross-database comparison.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CanonicalType {
    // Integers
    TinyInt,
    SmallInt,
    MediumInt,
    Int,
    BigInt,

    // Floating-point / exact numeric
    Float,
    Double,
    Decimal { precision: u32, scale: u32 },

    // Strings
    Char(u32),
    Varchar(u32),
    TinyText,
    Text,
    MediumText,
    LongText,

    // Binary
    Binary(u32),
    Varbinary(u32),
    TinyBlob,
    Blob,
    MediumBlob,
    LongBlob,

    // Date & time
    Date,
    Time { fsp: u8 },
    DateTime { fsp: u8 },
    Timestamp { fsp: u8 },
    Year,

    // JSON
    Json,
    Jsonb,

    // Boolean
    Boolean,

    // Enum / Set
    Enum(Vec<String>),
    Set(Vec<String>),

    // Network
    Inet,
    Inet6,

    // UUID
    Uuid,

    // Spatial
    Geometry,
    Point,
    LineString,
    Polygon,

    // Array (PostgreSQL)
    Array(Box<CanonicalType>),

    // Unmappable — stores original type string
    Unknown(String),
}
