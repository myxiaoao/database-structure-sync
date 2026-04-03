use crate::types::canonical::CanonicalType;
use crate::types::mapping::{TypeMapper, TypeMapping};
use crate::types::mysql_mapper::MySqlTypeMapper;

pub struct MariaDbTypeMapper;

impl TypeMapper for MariaDbTypeMapper {
    fn to_canonical(&self, raw_type: &str) -> CanonicalType {
        let lower = raw_type.trim().to_lowercase();
        match lower.as_str() {
            "uuid" => return CanonicalType::Uuid,
            "inet6" => return CanonicalType::Inet6,
            _ => {}
        }
        MySqlTypeMapper.to_canonical(raw_type)
    }

    fn from_canonical(&self, canonical: &CanonicalType) -> TypeMapping {
        match canonical {
            CanonicalType::Uuid => TypeMapping::direct("uuid"),
            CanonicalType::Inet6 => TypeMapping::direct("inet6"),
            CanonicalType::Jsonb => TypeMapping::degraded(
                "longtext",
                "jsonb → longtext: MariaDB JSON is longtext alias",
            ),
            _ => MySqlTypeMapper.from_canonical(canonical),
        }
    }

    fn map_default_value(&self, default: &str, canonical: &CanonicalType) -> Option<String> {
        MySqlTypeMapper.map_default_value(default, canonical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mariadb_uuid() {
        let m = MariaDbTypeMapper;
        assert_eq!(m.to_canonical("uuid"), CanonicalType::Uuid);
    }

    #[test]
    fn test_mariadb_inet6() {
        let m = MariaDbTypeMapper;
        assert_eq!(m.to_canonical("inet6"), CanonicalType::Inet6);
    }

    #[test]
    fn test_mariadb_json_is_json() {
        let m = MariaDbTypeMapper;
        assert_eq!(m.to_canonical("json"), CanonicalType::Json);
    }

    #[test]
    fn test_mariadb_inherits_mysql_basics() {
        let m = MariaDbTypeMapper;
        assert_eq!(m.to_canonical("int(11)"), CanonicalType::Int);
        assert_eq!(m.to_canonical("varchar(255)"), CanonicalType::Varchar(255));
        assert_eq!(m.to_canonical("tinyint(1)"), CanonicalType::Boolean);
    }

    #[test]
    fn test_from_canonical_uuid_native() {
        let m = MariaDbTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Uuid);
        assert_eq!(mapping.sql_type, "uuid");
        assert!(mapping.warning.is_none());
    }

    #[test]
    fn test_from_canonical_inet6_native() {
        let m = MariaDbTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Inet6);
        assert_eq!(mapping.sql_type, "inet6");
        assert!(mapping.warning.is_none());
    }

    #[test]
    fn test_from_canonical_jsonb_to_longtext() {
        let m = MariaDbTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Jsonb);
        assert_eq!(mapping.sql_type, "longtext");
        assert!(mapping.warning.is_some());
    }

    #[test]
    fn test_from_canonical_fallback_to_mysql() {
        let m = MariaDbTypeMapper;
        let mapping = m.from_canonical(&CanonicalType::Int);
        assert_eq!(mapping.sql_type, "int");
        assert!(mapping.warning.is_none());
    }
}
