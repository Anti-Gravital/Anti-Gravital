//! Generador SQL para DSL v0.1.
//!
//! Produce una migracion SQL `CREATE TABLE IF NOT EXISTS` idempotente
//! para cada modelo del schema. Compatible con PostgreSQL 14+.

use crate::ast::{Annotation, FieldDef, FieldType, ModelDef, Schema};

/// Genera el contenido del archivo de migracion SQL inicial.
///
/// El archivo resultante es idempotente: `CREATE TABLE IF NOT EXISTS` y
/// `CREATE UNIQUE INDEX IF NOT EXISTS` no fallan si la tabla ya existe.
pub fn generate_migration(schema: &Schema) -> String {
    let mut out = String::new();

    out.push_str("-- Migracion generada por Anti-Gravital ag-dsl v0.1.\n");
    out.push_str("-- NO editar manualmente. Regenerar con `ag generate`.\n\n");

    // Extension para gen_random_uuid() si hay campos UUID @auto
    let needs_uuid_ext = schema.models.iter().any(|m| {
        m.fields.iter().any(|f| {
            f.ty.value == FieldType::Uuid
                && f.annotations.iter().any(|a| a.value == Annotation::Auto)
        })
    });

    if needs_uuid_ext {
        out.push_str("CREATE EXTENSION IF NOT EXISTS \"pgcrypto\";\n\n");
    }

    for model in &schema.models {
        out.push_str(&generate_table(model));
        out.push('\n');
    }

    out
}

/// Genera el CREATE TABLE para un modelo.
fn generate_table(model: &ModelDef) -> String {
    let table_name = to_snake_case(&model.name.value);
    let mut out = String::new();

    // Recopila CHECK constraints de validaciones v0.3
    let checks: Vec<String> = model
        .fields
        .iter()
        .flat_map(|f| field_check_constraints(&table_name, f))
        .collect();

    out.push_str(&format!("CREATE TABLE IF NOT EXISTS \"{table_name}\" (\n"));

    let total_items = model.fields.len() + checks.len();
    let mut item_idx = 0;

    for field in &model.fields {
        let col = generate_column(field);
        item_idx += 1;
        if item_idx < total_items {
            out.push_str(&format!("    {col},\n"));
        } else {
            out.push_str(&format!("    {col}\n"));
        }
    }
    for check in &checks {
        item_idx += 1;
        if item_idx < total_items {
            out.push_str(&format!("    {check},\n"));
        } else {
            out.push_str(&format!("    {check}\n"));
        }
    }

    out.push_str(");\n");

    // Indices UNIQUE separados (mas flexibles que UNIQUE inline).
    for field in &model.fields {
        if field
            .annotations
            .iter()
            .any(|a| a.value == Annotation::Unique)
            && !field
                .annotations
                .iter()
                .any(|a| a.value == Annotation::Primary)
        {
            let col_name = to_snake_case(&field.name.value);
            let idx_name = format!("idx_{table_name}_{col_name}_unique");
            out.push_str(&format!(
                "CREATE UNIQUE INDEX IF NOT EXISTS \"{idx_name}\" \
                 ON \"{table_name}\" (\"{col_name}\");\n"
            ));
        }
    }

    out
}

/// Genera la definicion de columna SQL para un campo.
fn generate_column(field: &FieldDef) -> String {
    let col_name = to_snake_case(&field.name.value);
    let is_primary = field
        .annotations
        .iter()
        .any(|a| a.value == Annotation::Primary);
    let is_auto = field
        .annotations
        .iter()
        .any(|a| a.value == Annotation::Auto);
    let is_auto_update = field
        .annotations
        .iter()
        .any(|a| a.value == Annotation::AutoUpdate);
    let nullable = field.optional;

    // Tipo SQL base
    let sql_type = match (&field.ty.value, is_auto) {
        (FieldType::Int, true) => "BIGSERIAL",
        (ty, _) => ty.sql_type(),
    };

    let mut col = format!("\"{col_name}\" {sql_type}");

    // NOT NULL
    if !nullable && sql_type != "BIGSERIAL" {
        col.push_str(" NOT NULL");
    }

    // DEFAULT para campos @auto
    if is_auto && field.ty.value == FieldType::Uuid {
        col.push_str(" DEFAULT gen_random_uuid()");
    } else if is_auto && field.ty.value == FieldType::Timestamp {
        col.push_str(" DEFAULT NOW()");
    } else if is_auto_update {
        // @auto_update se maneja con trigger; no hay DEFAULT SQL directo.
        // La migracion incluye una nota al respecto.
        col.push_str(" DEFAULT NOW()");
    } else {
        // @default(valor)
        for ann in &field.annotations {
            if let Annotation::Default(ref val) = ann.value {
                col.push_str(&format!(" DEFAULT {val}"));
                break;
            }
        }
    }

    // PRIMARY KEY
    if is_primary {
        col.push_str(" PRIMARY KEY");
    }

    col
}

/// Delega a la utilidad de conversion en `ast`.
fn to_snake_case(s: &str) -> String {
    crate::ast::to_snake_case(s)
}

// ---- Validaciones v0.3 — CHECK constraints ----

/// Genera las clausulas CHECK para las anotaciones de validacion del campo.
fn field_check_constraints(table_name: &str, field: &FieldDef) -> Vec<String> {
    use crate::ast::FieldType;
    use Annotation::{Email, Length, Max, Min, Regex};

    let col_name = to_snake_case(&field.name.value);
    let mut checks = Vec::new();

    for ann in &field.annotations {
        let constraint_name = format!(
            "chk_{}_{}_{}",
            table_name,
            col_name,
            annotation_sql_suffix(&ann.value)
        );
        let expr = match &ann.value {
            Min(n) => match field.ty.value {
                FieldType::String => Some(format!("char_length(\"{col_name}\") >= {n}")),
                FieldType::Int | FieldType::Float | FieldType::Decimal => {
                    Some(format!("\"{col_name}\" >= {n}"))
                }
                _ => None,
            },
            Max(n) => match field.ty.value {
                FieldType::String => Some(format!("char_length(\"{col_name}\") <= {n}")),
                FieldType::Int | FieldType::Float | FieldType::Decimal => {
                    Some(format!("\"{col_name}\" <= {n}"))
                }
                _ => None,
            },
            Length(n) => Some(format!("char_length(\"{col_name}\") = {n}")),
            Regex(pattern) => {
                // Usa el operador ~ de PostgreSQL (regex POSIX).
                let escaped = pattern.replace('\'', "''");
                Some(format!("\"{col_name}\" ~ '{escaped}'"))
            }
            Email => {
                // Validacion basica: contiene @ y al menos un punto despues.
                Some(format!("\"{col_name}\" ~ '^[^@]+@[^@]+\\.[^@]+$'"))
            }
            _ => None,
        };
        if let Some(expr) = expr {
            checks.push(format!("CONSTRAINT \"{constraint_name}\" CHECK ({expr})"));
        }
    }
    checks
}

fn annotation_sql_suffix(ann: &Annotation) -> &'static str {
    match ann {
        Annotation::Min(_) => "min",
        Annotation::Max(_) => "max",
        Annotation::Email => "email",
        Annotation::Regex(_) => "regex",
        Annotation::Length(_) => "length",
        _ => "constraint",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse_tokens;

    fn schema_from(src: &str) -> Schema {
        let (tokens, _) = tokenize(src);
        let (ast, _) = parse_tokens(tokens, src.len());
        ast.expect("valid schema")
    }

    #[test]
    fn generates_create_table() {
        let schema = schema_from(
            r#"
model User {
    id    UUID   @primary @auto
    email String @unique
    name  String
}
"#,
        );
        let sql = generate_migration(&schema);
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"user\""));
        assert!(sql.contains("\"id\" UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY"));
        assert!(sql.contains("\"email\" TEXT NOT NULL"));
        assert!(sql.contains("\"name\" TEXT NOT NULL"));
    }

    #[test]
    fn unique_field_gets_index() {
        let schema = schema_from(
            r#"
model User {
    id    UUID   @primary @auto
    email String @unique
}
"#,
        );
        let sql = generate_migration(&schema);
        assert!(sql.contains("CREATE UNIQUE INDEX IF NOT EXISTS \"idx_user_email_unique\""));
    }

    #[test]
    fn optional_field_has_no_not_null() {
        let schema = schema_from(
            r#"
model Profile {
    id  UUID   @primary @auto
    bio String?
}
"#,
        );
        let sql = generate_migration(&schema);
        // El campo bio es nullable: no debe tener NOT NULL
        let bio_line = sql
            .lines()
            .find(|l| l.contains("\"bio\""))
            .expect("bio column missing");
        assert!(
            !bio_line.contains("NOT NULL"),
            "nullable field should not have NOT NULL: {bio_line}"
        );
    }

    #[test]
    fn int_auto_is_bigserial() {
        let schema = schema_from(
            r#"
model Counter {
    id    Int @primary @auto
    value Int
}
"#,
        );
        let sql = generate_migration(&schema);
        assert!(sql.contains("\"id\" BIGSERIAL"));
    }

    #[test]
    fn uuid_auto_includes_pgcrypto() {
        let schema = schema_from(
            r#"
model Thing {
    id UUID @primary @auto
    v  Int
}
"#,
        );
        let sql = generate_migration(&schema);
        assert!(sql.contains("CREATE EXTENSION IF NOT EXISTS \"pgcrypto\""));
    }

    #[test]
    fn to_snake_case_conversion() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("BlogPost"), "blog_post");
        assert_eq!(to_snake_case("HTTPRequest"), "h_t_t_p_request");
        assert_eq!(to_snake_case("id"), "id");
    }

    // ---- Tests DSL v0.3 ----

    #[test]
    fn v03_min_max_generates_check_constraints() {
        let schema = schema_from(
            r#"
model User {
    id    UUID   @primary @auto
    email String @max(255) @min(1)
}
"#,
        );
        let sql = generate_migration(&schema);
        assert!(
            sql.contains("CHECK (char_length(\"email\") <= 255)"),
            "should have max length check"
        );
        assert!(
            sql.contains("CHECK (char_length(\"email\") >= 1)"),
            "should have min length check"
        );
    }

    #[test]
    fn v03_email_generates_regex_check() {
        let schema = schema_from(
            r#"
model Contact {
    id    UUID @primary @auto
    email String @email
}
"#,
        );
        let sql = generate_migration(&schema);
        assert!(
            sql.contains("~ '^[^@]+@[^@]+\\.[^@]+$'"),
            "should have email regex check"
        );
    }

    #[test]
    fn v03_length_generates_exact_check() {
        let schema = schema_from(
            r#"
model Country {
    id   UUID @primary @auto
    code String @length(3)
}
"#,
        );
        let sql = generate_migration(&schema);
        assert!(
            sql.contains("char_length(\"code\") = 3"),
            "should have exact length check"
        );
    }

    #[test]
    fn v03_int_min_max_value_check() {
        let schema = schema_from(
            r#"
model Score {
    id    UUID @primary @auto
    value Int  @min(0) @max(100)
}
"#,
        );
        let sql = generate_migration(&schema);
        assert!(
            sql.contains("\"value\" >= 0"),
            "should have min value check"
        );
        assert!(
            sql.contains("\"value\" <= 100"),
            "should have max value check"
        );
    }
}
