use crate::types::canonical::CanonicalType;
use crate::types::mapping::{TypeMapper, TypeMapping};

pub struct PostgresTypeMapper;

fn parse_pg_width(s: &str) -> Option<u32> {
    let start = s.find('(')?;
    let end = s.find(')')?;
    s[start + 1..end].trim().parse().ok()
}

fn parse_pg_precision_scale(s: &str) -> Option<(u32, u32)> {
    let start = s.find('(')?;
    let end = s.find(')')?;
    let inner = &s[start + 1..end];
    let mut parts = inner.split(',');
    let p = parts.next()?.trim().parse().ok()?;
    let s = parts.next()?.trim().parse().ok()?;
    Some((p, s))
}

impl TypeMapper for PostgresTypeMapper {
    fn to_canonical(&self, raw_type: &str) -> CanonicalType {
        let lower = raw_type.trim().to_lowercase();

        if lower.ends_with("[]") {
            let inner = &lower[..lower.len() - 2];
            return CanonicalType::Array(Box::new(self.to_canonical(inner)));
        }

        if lower.starts_with("character varying") {
            let n = parse_pg_width(&lower).unwrap_or(255);
            return CanonicalType::Varchar(n);
        }
        if lower.starts_with("character") && !lower.starts_with("character varying") {
            let n = parse_pg_width(&lower).unwrap_or(1);
            return CanonicalType::Char(n);
        }
        if lower.starts_with("timestamp") {
            let fsp = parse_pg_width(&lower).unwrap_or(0) as u8;
            return CanonicalType::Timestamp { fsp };
        }
        if lower.starts_with("time") && !lower.starts_with("timestamp") {
            let fsp = parse_pg_width(&lower).unwrap_or(0) as u8;
            return CanonicalType::Time { fsp };
        }
        if lower.starts_with("double precision") {
            return CanonicalType::Double;
        }

        let base = lower.split('(').next().unwrap_or(&lower).trim();

        match base {
            "boolean" | "bool" => CanonicalType::Boolean,
            "smallint" | "int2" => CanonicalType::SmallInt,
            "integer" | "int" | "int4" => CanonicalType::Int,
            "bigint" | "int8" => CanonicalType::BigInt,
            "real" | "float4" => CanonicalType::Float,
            "float8" => CanonicalType::Double,
            "numeric" | "decimal" => {
                let (p, s) = parse_pg_precision_scale(&lower).unwrap_or((10, 0));
                CanonicalType::Decimal {
                    precision: p,
                    scale: s,
                }
            }
            "varchar" => {
                let n = parse_pg_width(&lower).unwrap_or(255);
                CanonicalType::Varchar(n)
            }
            "char" => {
                let n = parse_pg_width(&lower).unwrap_or(1);
                CanonicalType::Char(n)
            }
            "text" => CanonicalType::Text,
            "bytea" => CanonicalType::Blob,
            "date" => CanonicalType::Date,
            "json" => CanonicalType::Json,
            "jsonb" => CanonicalType::Jsonb,
            "uuid" => CanonicalType::Uuid,
            "inet" => CanonicalType::Inet,
            "serial" => CanonicalType::Int,
            "bigserial" => CanonicalType::BigInt,
            "smallserial" => CanonicalType::SmallInt,
            "geometry" => CanonicalType::Geometry,
            "point" => CanonicalType::Point,
            _ => CanonicalType::Unknown(raw_type.to_string()),
        }
    }

    fn from_canonical(&self, canonical: &CanonicalType) -> TypeMapping {
        match canonical {
            CanonicalType::TinyInt => {
                TypeMapping::degraded("smallint", "tinyint → smallint: range differs")
            }
            CanonicalType::SmallInt => TypeMapping::direct("smallint"),
            CanonicalType::MediumInt => {
                TypeMapping::degraded("integer", "mediumint → integer: range wider")
            }
            CanonicalType::Int => TypeMapping::direct("integer"),
            CanonicalType::BigInt => TypeMapping::direct("bigint"),
            CanonicalType::Float => TypeMapping::direct("real"),
            CanonicalType::Double => TypeMapping::direct("double precision"),
            CanonicalType::Decimal { precision, scale } => {
                TypeMapping::direct(format!("numeric({},{})", precision, scale))
            }
            CanonicalType::Char(n) => TypeMapping::direct(format!("character({})", n)),
            CanonicalType::Varchar(n) => TypeMapping::direct(format!("character varying({})", n)),
            CanonicalType::TinyText => {
                TypeMapping::degraded("text", "tinytext → text: no size limit distinction")
            }
            CanonicalType::Text => TypeMapping::direct("text"),
            CanonicalType::MediumText => {
                TypeMapping::degraded("text", "mediumtext → text: no size limit distinction")
            }
            CanonicalType::LongText => {
                TypeMapping::degraded("text", "longtext → text: no size limit distinction")
            }
            CanonicalType::Binary(_) | CanonicalType::Varbinary(_) => TypeMapping::direct("bytea"),
            CanonicalType::TinyBlob => {
                TypeMapping::degraded("bytea", "tinyblob → bytea: no size limit distinction")
            }
            CanonicalType::Blob => TypeMapping::direct("bytea"),
            CanonicalType::MediumBlob => {
                TypeMapping::degraded("bytea", "mediumblob → bytea: no size limit distinction")
            }
            CanonicalType::LongBlob => {
                TypeMapping::degraded("bytea", "longblob → bytea: no size limit distinction")
            }
            CanonicalType::Date => TypeMapping::direct("date"),
            CanonicalType::Time { fsp } => {
                if *fsp > 0 {
                    TypeMapping::direct(format!("time({})", fsp))
                } else {
                    TypeMapping::direct("time")
                }
            }
            CanonicalType::DateTime { fsp } | CanonicalType::Timestamp { fsp } => {
                if *fsp > 0 {
                    TypeMapping::direct(format!("timestamp({})", fsp))
                } else {
                    TypeMapping::direct("timestamp")
                }
            }
            CanonicalType::Year => {
                TypeMapping::degraded("smallint", "year → smallint: no native year type")
            }
            CanonicalType::Json => TypeMapping::direct("json"),
            CanonicalType::Jsonb => TypeMapping::direct("jsonb"),
            CanonicalType::Boolean => TypeMapping::direct("boolean"),
            CanonicalType::Enum(values) => {
                let max_len = values.iter().map(|v| v.len()).max().unwrap_or(255);
                TypeMapping::degraded(
                    format!("character varying({})", max_len.max(255)),
                    "enum → varchar: PostgreSQL requires CREATE TYPE for native enums",
                )
            }
            CanonicalType::Set(values) => {
                let _ = values;
                TypeMapping::degraded("text[]", "set → text[]: different semantics")
            }
            CanonicalType::Inet => TypeMapping::direct("inet"),
            CanonicalType::Inet6 => {
                TypeMapping::degraded("inet", "inet6 → inet: IPv6-specific features may differ")
            }
            CanonicalType::Uuid => TypeMapping::direct("uuid"),
            CanonicalType::Geometry => TypeMapping::direct("geometry"),
            CanonicalType::Point => TypeMapping::direct("point"),
            CanonicalType::LineString => TypeMapping::degraded(
                "geometry",
                "linestring → geometry: specific type unavailable without PostGIS",
            ),
            CanonicalType::Polygon => TypeMapping::degraded(
                "geometry",
                "polygon → geometry: specific type unavailable without PostGIS",
            ),
            CanonicalType::Array(inner) => {
                let inner_mapping = self.from_canonical(inner);
                TypeMapping::direct(format!("{}[]", inner_mapping.sql_type))
            }
            CanonicalType::Unknown(s) => {
                TypeMapping::skipped(format!("Cannot map type '{}' to PostgreSQL", s))
            }
        }
    }

    fn map_default_value(&self, default: &str, canonical: &CanonicalType) -> Option<String> {
        let trimmed = default.trim();
        if matches!(canonical, CanonicalType::Boolean) {
            return match trimmed {
                "1" | "b'1'" => Some("true".to_string()),
                "0" | "b'0'" => Some("false".to_string()),
                "true" | "false" => Some(trimmed.to_string()),
                _ => Some(trimmed.to_string()),
            };
        }
        if trimmed.eq_ignore_ascii_case("CURRENT_TIMESTAMP") {
            return Some("CURRENT_TIMESTAMP".to_string());
        }
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_types() {
        let m = PostgresTypeMapper;
        assert_eq!(m.to_canonical("integer"), CanonicalType::Int);
        assert_eq!(m.to_canonical("int4"), CanonicalType::Int);
        assert_eq!(m.to_canonical("bigint"), CanonicalType::BigInt);
        assert_eq!(m.to_canonical("int8"), CanonicalType::BigInt);
        assert_eq!(m.to_canonical("smallint"), CanonicalType::SmallInt);
        assert_eq!(m.to_canonical("int2"), CanonicalType::SmallInt);
    }

    #[test]
    fn test_boolean() {
        let m = PostgresTypeMapper;
        assert_eq!(m.to_canonical("boolean"), CanonicalType::Boolean);
        assert_eq!(m.to_canonical("bool"), CanonicalType::Boolean);
    }

    #[test]
    fn test_string_types() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.to_canonical("character varying(255)"),
            CanonicalType::Varchar(255)
        );
        assert_eq!(m.to_canonical("varchar(100)"), CanonicalType::Varchar(100));
        assert_eq!(m.to_canonical("character(36)"), CanonicalType::Char(36));
        assert_eq!(m.to_canonical("char(10)"), CanonicalType::Char(10));
        assert_eq!(m.to_canonical("text"), CanonicalType::Text);
    }

    #[test]
    fn test_numeric() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.to_canonical("numeric(10,2)"),
            CanonicalType::Decimal {
                precision: 10,
                scale: 2
            }
        );
        assert_eq!(
            m.to_canonical("decimal(5,3)"),
            CanonicalType::Decimal {
                precision: 5,
                scale: 3
            }
        );
    }

    #[test]
    fn test_float_types() {
        let m = PostgresTypeMapper;
        assert_eq!(m.to_canonical("real"), CanonicalType::Float);
        assert_eq!(m.to_canonical("float4"), CanonicalType::Float);
        assert_eq!(m.to_canonical("double precision"), CanonicalType::Double);
        assert_eq!(m.to_canonical("float8"), CanonicalType::Double);
    }

    #[test]
    fn test_datetime_types() {
        let m = PostgresTypeMapper;
        assert_eq!(m.to_canonical("date"), CanonicalType::Date);
        assert_eq!(
            m.to_canonical("timestamp"),
            CanonicalType::Timestamp { fsp: 0 }
        );
        assert_eq!(
            m.to_canonical("timestamp(3)"),
            CanonicalType::Timestamp { fsp: 3 }
        );
        assert_eq!(
            m.to_canonical("timestamp without time zone"),
            CanonicalType::Timestamp { fsp: 0 }
        );
        assert_eq!(m.to_canonical("time"), CanonicalType::Time { fsp: 0 });
    }

    #[test]
    fn test_json_types() {
        let m = PostgresTypeMapper;
        assert_eq!(m.to_canonical("json"), CanonicalType::Json);
        assert_eq!(m.to_canonical("jsonb"), CanonicalType::Jsonb);
    }

    #[test]
    fn test_special_types() {
        let m = PostgresTypeMapper;
        assert_eq!(m.to_canonical("uuid"), CanonicalType::Uuid);
        assert_eq!(m.to_canonical("inet"), CanonicalType::Inet);
        assert_eq!(m.to_canonical("bytea"), CanonicalType::Blob);
    }

    #[test]
    fn test_serial_types() {
        let m = PostgresTypeMapper;
        assert_eq!(m.to_canonical("serial"), CanonicalType::Int);
        assert_eq!(m.to_canonical("bigserial"), CanonicalType::BigInt);
    }

    #[test]
    fn test_array_types() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.to_canonical("integer[]"),
            CanonicalType::Array(Box::new(CanonicalType::Int))
        );
        assert_eq!(
            m.to_canonical("text[]"),
            CanonicalType::Array(Box::new(CanonicalType::Text))
        );
    }

    #[test]
    fn test_from_canonical_basic() {
        let m = PostgresTypeMapper;
        assert_eq!(m.from_canonical(&CanonicalType::Int).sql_type, "integer");
        assert_eq!(
            m.from_canonical(&CanonicalType::Boolean).sql_type,
            "boolean"
        );
        assert_eq!(m.from_canonical(&CanonicalType::Text).sql_type, "text");
    }

    #[test]
    fn test_from_canonical_degraded() {
        let m = PostgresTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::TinyInt);
        assert_eq!(mapping.sql_type, "smallint");
        assert!(mapping.warning.is_some());
        let mapping = m.from_canonical(&CanonicalType::MediumText);
        assert_eq!(mapping.sql_type, "text");
        assert!(mapping.warning.is_some());
        let mapping = m.from_canonical(&CanonicalType::Year);
        assert_eq!(mapping.sql_type, "smallint");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_set_degraded() {
        let m = PostgresTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Set(vec!["a".into(), "b".into()]));
        assert_eq!(mapping.sql_type, "text[]");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_map_default_value_boolean() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.map_default_value("1", &CanonicalType::Boolean),
            Some("true".to_string())
        );
        assert_eq!(
            m.map_default_value("0", &CanonicalType::Boolean),
            Some("false".to_string())
        );
    }

    #[test]
    fn test_map_default_value_current_timestamp() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.map_default_value("CURRENT_TIMESTAMP", &CanonicalType::Timestamp { fsp: 0 }),
            Some("CURRENT_TIMESTAMP".to_string())
        );
    }

    #[test]
    fn test_map_default_value_bit_literal() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.map_default_value("b'0'", &CanonicalType::Boolean),
            Some("false".to_string())
        );
        assert_eq!(
            m.map_default_value("b'1'", &CanonicalType::Boolean),
            Some("true".to_string())
        );
    }

    #[test]
    fn test_from_canonical_binary_varbinary_to_bytea() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::Binary(16)).sql_type,
            "bytea"
        );
        assert!(
            m.from_canonical(&CanonicalType::Binary(16))
                .warning
                .is_none()
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Varbinary(256)).sql_type,
            "bytea"
        );
    }

    #[test]
    fn test_from_canonical_blob_variants() {
        let m = PostgresTypeMapper;
        let tiny = m.from_canonical(&CanonicalType::TinyBlob);
        assert_eq!(tiny.sql_type, "bytea");
        assert!(tiny.warning.is_some());

        let medium = m.from_canonical(&CanonicalType::MediumBlob);
        assert_eq!(medium.sql_type, "bytea");
        assert!(medium.warning.is_some());

        let long = m.from_canonical(&CanonicalType::LongBlob);
        assert_eq!(long.sql_type, "bytea");
        assert!(long.warning.is_some());

        let blob = m.from_canonical(&CanonicalType::Blob);
        assert_eq!(blob.sql_type, "bytea");
        assert!(blob.warning.is_none());
    }

    #[test]
    fn test_from_canonical_longtext_degraded() {
        let m = PostgresTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::LongText);
        assert_eq!(mapping.sql_type, "text");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_tinytext_degraded() {
        let m = PostgresTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::TinyText);
        assert_eq!(mapping.sql_type, "text");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_time_with_fsp() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::Time { fsp: 3 }).sql_type,
            "time(3)"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Time { fsp: 0 }).sql_type,
            "time"
        );
    }

    #[test]
    fn test_from_canonical_datetime_timestamp_with_fsp() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::DateTime { fsp: 6 })
                .sql_type,
            "timestamp(6)"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::DateTime { fsp: 0 })
                .sql_type,
            "timestamp"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Timestamp { fsp: 3 })
                .sql_type,
            "timestamp(3)"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Timestamp { fsp: 0 })
                .sql_type,
            "timestamp"
        );
    }

    #[test]
    fn test_from_canonical_enum_degraded() {
        let m = PostgresTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Enum(vec![
            "active".into(),
            "inactive".into(),
        ]));
        assert!(mapping.sql_type.starts_with("character varying"));
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_inet6_degraded() {
        let m = PostgresTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Inet6);
        assert_eq!(mapping.sql_type, "inet");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_linestring_polygon_degraded() {
        let m = PostgresTypeMapper;
        let ls = m.from_canonical(&CanonicalType::LineString);
        assert_eq!(ls.sql_type, "geometry");
        assert!(ls.warning.is_some());

        let pg = m.from_canonical(&CanonicalType::Polygon);
        assert_eq!(pg.sql_type, "geometry");
        assert!(pg.warning.is_some());
    }

    #[test]
    fn test_from_canonical_array_recursive() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::Array(Box::new(CanonicalType::Int)))
                .sql_type,
            "integer[]"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Array(Box::new(CanonicalType::Text)))
                .sql_type,
            "text[]"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Array(Box::new(CanonicalType::Boolean)))
                .sql_type,
            "boolean[]"
        );
    }

    #[test]
    fn test_from_canonical_unknown_skipped() {
        let m = PostgresTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Unknown("hstore".to_string()));
        assert!(mapping.skipped);
    }

    #[test]
    fn test_from_canonical_decimal() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::Decimal {
                precision: 10,
                scale: 2
            })
            .sql_type,
            "numeric(10,2)"
        );
    }

    #[test]
    fn test_from_canonical_char_varchar() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::Char(36)).sql_type,
            "character(36)"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Varchar(255)).sql_type,
            "character varying(255)"
        );
    }

    #[test]
    fn test_map_default_value_passthrough() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.map_default_value("'test'", &CanonicalType::Varchar(255)),
            Some("'test'".to_string())
        );
        assert_eq!(
            m.map_default_value("42", &CanonicalType::Int),
            Some("42".to_string())
        );
    }

    #[test]
    fn test_map_default_value_whitespace_trim() {
        let m = PostgresTypeMapper;
        assert_eq!(
            m.map_default_value(
                "  CURRENT_TIMESTAMP  ",
                &CanonicalType::Timestamp { fsp: 0 }
            ),
            Some("CURRENT_TIMESTAMP".to_string())
        );
    }
}
