use crate::types::canonical::CanonicalType;
use crate::types::mapping::{TypeMapper, TypeMapping};

pub struct MySqlTypeMapper;

/// Parse the display width from types like "int(11)", "tinyint(1)".
fn parse_display_width(s: &str) -> Option<u32> {
    let start = s.find('(')?;
    let end = s.find(')')?;
    s[start + 1..end].parse().ok()
}

/// Parse precision and scale from types like "decimal(10,2)".
fn parse_precision_scale(s: &str) -> Option<(u32, u32)> {
    let start = s.find('(')?;
    let end = s.find(')')?;
    let inner = &s[start + 1..end];
    let mut parts = inner.split(',');
    let p = parts.next()?.trim().parse().ok()?;
    let s = parts.next()?.trim().parse().ok()?;
    Some((p, s))
}

/// Parse enum/set values from "enum('a','b','c')" or "set('x','y')".
fn parse_enum_values(s: &str) -> Vec<String> {
    let start = match s.find('(') {
        Some(i) => i + 1,
        None => return vec![],
    };
    let end = match s.rfind(')') {
        Some(i) => i,
        None => return vec![],
    };
    let inner = &s[start..end];
    inner
        .split(',')
        .map(|v| v.trim().trim_matches('\'').to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

impl TypeMapper for MySqlTypeMapper {
    fn to_canonical(&self, raw_type: &str) -> CanonicalType {
        let lower = raw_type.trim().to_lowercase();
        let base = lower.split('(').next().unwrap_or(&lower).trim();

        match base {
            "tinyint" => {
                if parse_display_width(&lower) == Some(1) {
                    CanonicalType::Boolean
                } else {
                    CanonicalType::TinyInt
                }
            }
            "smallint" => CanonicalType::SmallInt,
            "mediumint" => CanonicalType::MediumInt,
            "int" | "integer" => CanonicalType::Int,
            "bigint" => CanonicalType::BigInt,
            "float" => CanonicalType::Float,
            "double" => CanonicalType::Double,
            "decimal" | "numeric" => {
                let (p, s) = parse_precision_scale(&lower).unwrap_or((10, 0));
                CanonicalType::Decimal {
                    precision: p,
                    scale: s,
                }
            }
            "char" => {
                let n = parse_display_width(&lower).unwrap_or(1);
                CanonicalType::Char(n)
            }
            "varchar" => {
                let n = parse_display_width(&lower).unwrap_or(255);
                CanonicalType::Varchar(n)
            }
            "tinytext" => CanonicalType::TinyText,
            "text" => CanonicalType::Text,
            "mediumtext" => CanonicalType::MediumText,
            "longtext" => CanonicalType::LongText,
            "binary" => {
                let n = parse_display_width(&lower).unwrap_or(1);
                CanonicalType::Binary(n)
            }
            "varbinary" => {
                let n = parse_display_width(&lower).unwrap_or(255);
                CanonicalType::Varbinary(n)
            }
            "tinyblob" => CanonicalType::TinyBlob,
            "blob" => CanonicalType::Blob,
            "mediumblob" => CanonicalType::MediumBlob,
            "longblob" => CanonicalType::LongBlob,
            "date" => CanonicalType::Date,
            "time" => {
                let fsp = parse_display_width(&lower).unwrap_or(0) as u8;
                CanonicalType::Time { fsp }
            }
            "datetime" => {
                let fsp = parse_display_width(&lower).unwrap_or(0) as u8;
                CanonicalType::DateTime { fsp }
            }
            "timestamp" => {
                let fsp = parse_display_width(&lower).unwrap_or(0) as u8;
                CanonicalType::Timestamp { fsp }
            }
            "year" => CanonicalType::Year,
            "json" => CanonicalType::Json,
            "enum" => CanonicalType::Enum(parse_enum_values(&lower)),
            "set" => CanonicalType::Set(parse_enum_values(&lower)),
            "geometry" => CanonicalType::Geometry,
            "point" => CanonicalType::Point,
            "linestring" => CanonicalType::LineString,
            "polygon" => CanonicalType::Polygon,
            _ => CanonicalType::Unknown(raw_type.to_string()),
        }
    }

    fn from_canonical(&self, canonical: &CanonicalType) -> TypeMapping {
        match canonical {
            CanonicalType::TinyInt => TypeMapping::direct("tinyint"),
            CanonicalType::SmallInt => TypeMapping::direct("smallint"),
            CanonicalType::MediumInt => TypeMapping::direct("mediumint"),
            CanonicalType::Int => TypeMapping::direct("int"),
            CanonicalType::BigInt => TypeMapping::direct("bigint"),
            CanonicalType::Float => TypeMapping::direct("float"),
            CanonicalType::Double => TypeMapping::direct("double"),
            CanonicalType::Decimal { precision, scale } => {
                TypeMapping::direct(format!("decimal({},{})", precision, scale))
            }
            CanonicalType::Char(n) => TypeMapping::direct(format!("char({})", n)),
            CanonicalType::Varchar(n) => TypeMapping::direct(format!("varchar({})", n)),
            CanonicalType::TinyText => TypeMapping::direct("tinytext"),
            CanonicalType::Text => TypeMapping::direct("text"),
            CanonicalType::MediumText => TypeMapping::direct("mediumtext"),
            CanonicalType::LongText => TypeMapping::direct("longtext"),
            CanonicalType::Binary(n) => TypeMapping::direct(format!("binary({})", n)),
            CanonicalType::Varbinary(n) => TypeMapping::direct(format!("varbinary({})", n)),
            CanonicalType::TinyBlob => TypeMapping::direct("tinyblob"),
            CanonicalType::Blob => TypeMapping::direct("blob"),
            CanonicalType::MediumBlob => TypeMapping::direct("mediumblob"),
            CanonicalType::LongBlob => TypeMapping::direct("longblob"),
            CanonicalType::Date => TypeMapping::direct("date"),
            CanonicalType::Time { fsp } => {
                if *fsp > 0 {
                    TypeMapping::direct(format!("time({})", fsp))
                } else {
                    TypeMapping::direct("time")
                }
            }
            CanonicalType::DateTime { fsp } => {
                if *fsp > 0 {
                    TypeMapping::direct(format!("datetime({})", fsp))
                } else {
                    TypeMapping::direct("datetime")
                }
            }
            CanonicalType::Timestamp { fsp } => {
                if *fsp > 0 {
                    TypeMapping::direct(format!("timestamp({})", fsp))
                } else {
                    TypeMapping::direct("timestamp")
                }
            }
            CanonicalType::Year => TypeMapping::direct("year"),
            CanonicalType::Json => TypeMapping::direct("json"),
            CanonicalType::Jsonb => {
                TypeMapping::degraded("json", "jsonb → json: binary JSON features lost")
            }
            CanonicalType::Boolean => TypeMapping::direct("tinyint(1)"),
            CanonicalType::Enum(values) => {
                let vals = values
                    .iter()
                    .map(|v| format!("'{}'", v))
                    .collect::<Vec<_>>()
                    .join(",");
                TypeMapping::direct(format!("enum({})", vals))
            }
            CanonicalType::Set(values) => {
                let vals = values
                    .iter()
                    .map(|v| format!("'{}'", v))
                    .collect::<Vec<_>>()
                    .join(",");
                TypeMapping::direct(format!("set({})", vals))
            }
            CanonicalType::Inet => TypeMapping::degraded(
                "varchar(45)",
                "inet → varchar(45): no native inet type in MySQL",
            ),
            CanonicalType::Inet6 => TypeMapping::degraded(
                "varchar(45)",
                "inet6 → varchar(45): no native inet6 type in MySQL",
            ),
            CanonicalType::Uuid => {
                TypeMapping::degraded("char(36)", "uuid → char(36): no native UUID type in MySQL")
            }
            CanonicalType::Geometry => TypeMapping::direct("geometry"),
            CanonicalType::Point => TypeMapping::direct("point"),
            CanonicalType::LineString => TypeMapping::direct("linestring"),
            CanonicalType::Polygon => TypeMapping::direct("polygon"),
            CanonicalType::Array(_) => {
                TypeMapping::degraded("json", "array → json: array semantics lost")
            }
            CanonicalType::Unknown(s) => {
                TypeMapping::skipped(format!("Cannot map type '{}' to MySQL", s))
            }
        }
    }

    fn map_default_value(&self, default: &str, canonical: &CanonicalType) -> Option<String> {
        let trimmed = default.trim();
        if trimmed.eq_ignore_ascii_case("now()") {
            return Some("CURRENT_TIMESTAMP".to_string());
        }
        if matches!(canonical, CanonicalType::Boolean) {
            return match trimmed.to_lowercase().as_str() {
                "true" => Some("1".to_string()),
                "false" => Some("0".to_string()),
                _ => Some(trimmed.to_string()),
            };
        }
        if trimmed.starts_with("nextval(") {
            return None;
        }
        if let Some(pos) = trimmed.find("::") {
            return Some(trimmed[..pos].to_string());
        }
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_types() {
        let m = MySqlTypeMapper;
        assert_eq!(m.to_canonical("int"), CanonicalType::Int);
        assert_eq!(m.to_canonical("int(11)"), CanonicalType::Int);
        assert_eq!(m.to_canonical("bigint(20)"), CanonicalType::BigInt);
        assert_eq!(m.to_canonical("smallint(6)"), CanonicalType::SmallInt);
        assert_eq!(m.to_canonical("mediumint(9)"), CanonicalType::MediumInt);
    }

    #[test]
    fn test_tinyint_boolean() {
        let m = MySqlTypeMapper;
        assert_eq!(m.to_canonical("tinyint(1)"), CanonicalType::Boolean);
        assert_eq!(m.to_canonical("tinyint(4)"), CanonicalType::TinyInt);
        assert_eq!(m.to_canonical("tinyint"), CanonicalType::TinyInt);
    }

    #[test]
    fn test_string_types() {
        let m = MySqlTypeMapper;
        assert_eq!(m.to_canonical("varchar(255)"), CanonicalType::Varchar(255));
        assert_eq!(m.to_canonical("char(36)"), CanonicalType::Char(36));
        assert_eq!(m.to_canonical("text"), CanonicalType::Text);
        assert_eq!(m.to_canonical("longtext"), CanonicalType::LongText);
        assert_eq!(m.to_canonical("mediumtext"), CanonicalType::MediumText);
        assert_eq!(m.to_canonical("tinytext"), CanonicalType::TinyText);
    }

    #[test]
    fn test_decimal() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.to_canonical("decimal(10,2)"),
            CanonicalType::Decimal {
                precision: 10,
                scale: 2
            }
        );
    }

    #[test]
    fn test_datetime_types() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.to_canonical("datetime"),
            CanonicalType::DateTime { fsp: 0 }
        );
        assert_eq!(
            m.to_canonical("datetime(3)"),
            CanonicalType::DateTime { fsp: 3 }
        );
        assert_eq!(
            m.to_canonical("timestamp"),
            CanonicalType::Timestamp { fsp: 0 }
        );
        assert_eq!(
            m.to_canonical("timestamp(6)"),
            CanonicalType::Timestamp { fsp: 6 }
        );
        assert_eq!(m.to_canonical("date"), CanonicalType::Date);
        assert_eq!(m.to_canonical("time"), CanonicalType::Time { fsp: 0 });
        assert_eq!(m.to_canonical("year"), CanonicalType::Year);
    }

    #[test]
    fn test_json_and_special() {
        let m = MySqlTypeMapper;
        assert_eq!(m.to_canonical("json"), CanonicalType::Json);
        assert_eq!(m.to_canonical("float"), CanonicalType::Float);
        assert_eq!(m.to_canonical("double"), CanonicalType::Double);
    }

    #[test]
    fn test_binary_types() {
        let m = MySqlTypeMapper;
        assert_eq!(m.to_canonical("binary(16)"), CanonicalType::Binary(16));
        assert_eq!(
            m.to_canonical("varbinary(256)"),
            CanonicalType::Varbinary(256)
        );
        assert_eq!(m.to_canonical("blob"), CanonicalType::Blob);
        assert_eq!(m.to_canonical("longblob"), CanonicalType::LongBlob);
    }

    #[test]
    fn test_enum_set() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.to_canonical("enum('a','b','c')"),
            CanonicalType::Enum(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
        assert_eq!(
            m.to_canonical("set('x','y')"),
            CanonicalType::Set(vec!["x".to_string(), "y".to_string()])
        );
    }

    #[test]
    fn test_spatial() {
        let m = MySqlTypeMapper;
        assert_eq!(m.to_canonical("geometry"), CanonicalType::Geometry);
        assert_eq!(m.to_canonical("point"), CanonicalType::Point);
    }

    #[test]
    fn test_unknown() {
        let m = MySqlTypeMapper;
        assert!(matches!(
            m.to_canonical("some_custom_type"),
            CanonicalType::Unknown(_)
        ));
    }

    #[test]
    fn test_from_canonical_basic() {
        let m = MySqlTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Int);
        assert_eq!(mapping.sql_type, "int");
        assert!(mapping.warning.is_none());
        assert!(!mapping.skipped);
    }

    #[test]
    fn test_from_canonical_boolean() {
        let m = MySqlTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Boolean);
        assert_eq!(mapping.sql_type, "tinyint(1)");
    }

    #[test]
    fn test_from_canonical_jsonb_degraded() {
        let m = MySqlTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Jsonb);
        assert_eq!(mapping.sql_type, "json");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_uuid_degraded() {
        let m = MySqlTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Uuid);
        assert_eq!(mapping.sql_type, "char(36)");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_array_degraded() {
        let m = MySqlTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Array(Box::new(CanonicalType::Int)));
        assert_eq!(mapping.sql_type, "json");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_unknown_skipped() {
        let m = MySqlTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Unknown("hstore".to_string()));
        assert!(mapping.skipped);
    }

    #[test]
    fn test_map_default_value_boolean() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.map_default_value("true", &CanonicalType::Boolean),
            Some("1".to_string())
        );
        assert_eq!(
            m.map_default_value("false", &CanonicalType::Boolean),
            Some("0".to_string())
        );
    }

    #[test]
    fn test_map_default_value_now() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.map_default_value("now()", &CanonicalType::Timestamp { fsp: 0 }),
            Some("CURRENT_TIMESTAMP".to_string())
        );
    }

    #[test]
    fn test_map_default_value_passthrough() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.map_default_value("42", &CanonicalType::Int),
            Some("42".to_string())
        );
    }

    #[test]
    fn test_from_canonical_inet_degraded() {
        let m = MySqlTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Inet);
        assert_eq!(mapping.sql_type, "varchar(45)");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_inet6_degraded() {
        let m = MySqlTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Inet6);
        assert_eq!(mapping.sql_type, "varchar(45)");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_decimal() {
        let m = MySqlTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Decimal {
            precision: 10,
            scale: 2,
        });
        assert_eq!(mapping.sql_type, "decimal(10,2)");
        assert!(mapping.warning.is_none());
    }

    #[test]
    fn test_from_canonical_char_varchar() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::Char(36)).sql_type,
            "char(36)"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Varchar(255)).sql_type,
            "varchar(255)"
        );
    }

    #[test]
    fn test_from_canonical_binary_varbinary() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::Binary(16)).sql_type,
            "binary(16)"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Varbinary(256)).sql_type,
            "varbinary(256)"
        );
    }

    #[test]
    fn test_from_canonical_time_with_fsp() {
        let m = MySqlTypeMapper;
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
    fn test_from_canonical_datetime_with_fsp() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::DateTime { fsp: 6 })
                .sql_type,
            "datetime(6)"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::DateTime { fsp: 0 })
                .sql_type,
            "datetime"
        );
    }

    #[test]
    fn test_from_canonical_timestamp_with_fsp() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::Timestamp { fsp: 4 })
                .sql_type,
            "timestamp(4)"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Timestamp { fsp: 0 })
                .sql_type,
            "timestamp"
        );
    }

    #[test]
    fn test_from_canonical_enum_set() {
        let m = MySqlTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Enum(vec!["a".into(), "b".into()]));
        assert_eq!(mapping.sql_type, "enum('a','b')");
        assert!(mapping.warning.is_none());

        let mapping = m.from_canonical(&CanonicalType::Set(vec!["x".into()]));
        assert_eq!(mapping.sql_type, "set('x')");
        assert!(mapping.warning.is_none());
    }

    #[test]
    fn test_from_canonical_spatial() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.from_canonical(&CanonicalType::Geometry).sql_type,
            "geometry"
        );
        assert_eq!(m.from_canonical(&CanonicalType::Point).sql_type, "point");
        assert_eq!(
            m.from_canonical(&CanonicalType::LineString).sql_type,
            "linestring"
        );
        assert_eq!(
            m.from_canonical(&CanonicalType::Polygon).sql_type,
            "polygon"
        );
    }

    #[test]
    fn test_map_default_value_nextval_returns_none() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.map_default_value("nextval('users_id_seq'::regclass)", &CanonicalType::Int),
            None
        );
    }

    #[test]
    fn test_map_default_value_strips_pg_cast() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.map_default_value("'abc'::text", &CanonicalType::Varchar(255)),
            Some("'abc'".to_string())
        );
        assert_eq!(
            m.map_default_value("123::integer", &CanonicalType::Int),
            Some("123".to_string())
        );
    }

    #[test]
    fn test_map_default_value_now_case_insensitive() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.map_default_value("NOW()", &CanonicalType::DateTime { fsp: 0 }),
            Some("CURRENT_TIMESTAMP".to_string())
        );
        assert_eq!(
            m.map_default_value("Now()", &CanonicalType::DateTime { fsp: 0 }),
            Some("CURRENT_TIMESTAMP".to_string())
        );
    }

    #[test]
    fn test_map_default_value_whitespace_trim() {
        let m = MySqlTypeMapper;
        assert_eq!(
            m.map_default_value("  42  ", &CanonicalType::Int),
            Some("42".to_string())
        );
    }
}
