pub struct CustomBuilder;

impl CustomBuilder {
    pub fn secure_category(
        secure_category_ids: Option<&[i32]>,
        category_alias: Option<&str>,
    ) -> String {
        let alias = category_alias.unwrap_or("c");

        match secure_category_ids {
            Some(ids) if !ids.is_empty() => {
                let ids_str = ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!(
                    "NOT COALESCE({alias}.read_restricted, false) OR {alias}.id in ({ids_str})",
                    alias = alias,
                    ids_str = ids_str
                )
            }
            _ => format!(
                "NOT COALESCE({alias}.read_restricted, false)",
                alias = alias
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_category() {
        assert_eq!(
            CustomBuilder::secure_category(None, None),
            "NOT COALESCE(c.read_restricted, false)"
        );

        assert_eq!(
            CustomBuilder::secure_category(None, Some("cat")),
            "NOT COALESCE(cat.read_restricted, false)"
        );

        assert_eq!(
            CustomBuilder::secure_category(Some(&[]), None),
            "NOT COALESCE(c.read_restricted, false)"
        );

        assert_eq!(
            CustomBuilder::secure_category(Some(&[1, 2, 3]), None),
            "NOT COALESCE(c.read_restricted, false) OR c.id in (1, 2, 3)"
        );

        assert_eq!(
            CustomBuilder::secure_category(Some(&[4, 5]), Some("t")),
            "NOT COALESCE(t.read_restricted, false) OR t.id in (4, 5)"
        );
    }
}
