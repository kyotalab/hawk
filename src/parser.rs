use crate::Error;

pub fn parse_query_segments(query: &str) -> Result<(&str, Vec<&str>), Error> {
    let mut segments = query.split('.').skip(1);
    let segment = segments.next().ok_or(Error::InvalidQuery("Missing field segment in query".into()))?;

    let fields: Vec<&str> = segments.collect();

    if fields.is_empty() {
        return Err(Error::InvalidQuery("Missing parameter segment in query".into()));
    }

    Ok((segment, fields))
}

pub fn parse_array_segment(segment: &str) -> Result<(usize, usize), Error> {
    let idx = segment.find('[').ok_or(Error::InvalidQuery("Missing '[' in segment".into()))?;
    let ridx = segment.find(']').ok_or(Error::InvalidQuery("Missing ']' in segment".into()))?;

    if idx >= ridx {
        return Err(Error::InvalidQuery("Invalid bracket order".into()));
    }

    Ok((idx, ridx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn test_parse_query_segments_normal_case() {
        // 正常ケース: 基本的なクエリ
        let result = parse_query_segments(".users.name");
        assert!(result.is_ok());
        let (segment, field) = result.unwrap();
        assert_eq!(segment, "users");
        assert_eq!(field, vec!["name"]);
    }

    #[test]
    fn test_parse_query_segments_with_array_index() {
        // 正常ケース: 配列インデックス付き
        let result = parse_query_segments(".users[0].name");
        assert!(result.is_ok());
        let (segment, field) = result.unwrap();
        assert_eq!(segment, "users[0]");
        assert_eq!(field, vec!["name"]);
    }

    #[test]
    fn test_parse_query_segments_different_fields() {
        // 正常ケース: 異なるフィールド名
        let result = parse_query_segments(".products.price");
        assert!(result.is_ok());
        let (segment, field) = result.unwrap();
        assert_eq!(segment, "products");
        assert_eq!(field, vec!["price"]);
    }

    #[test]
    fn test_parse_query_segments_complex_index() {
        // 正常ケース: 大きなインデックス
        let result = parse_query_segments(".items[123].description");
        assert!(result.is_ok());
        let (segment, field) = result.unwrap();
        assert_eq!(segment, "items[123]");
        assert_eq!(field, vec!["description"]);
    }

    #[test]
    fn test_parse_query_segments_missing_field_segment() {
        // エラーケース: フィールドセグメントが不足
        let result = parse_query_segments(".");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidQuery(msg) => {
                assert!(msg.contains("Missing parameter segment"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }

    #[test]
    fn test_parse_query_segments_truly_missing_field() {
        // エラーケース: 本当にフィールドセグメントが不足
        let result = parse_query_segments("");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidQuery(msg) => {
                assert!(msg.contains("Missing field segment"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }

    #[test]
    fn test_parse_query_segments_missing_parameter_segment() {
        // エラーケース: パラメータセグメントが不足
        let result = parse_query_segments(".users");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidQuery(msg) => {
                assert!(msg.contains("Missing parameter segment"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }

    #[test]
    fn test_parse_query_segments_empty_query() {
        // エラーケース: 空のクエリ
        let result = parse_query_segments("");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidQuery(msg) => {
                assert!(msg.contains("Missing field segment"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }

    #[test]
    fn test_parse_query_segments_no_leading_dot() {
        // エラーケース: 先頭の.がない
        let result = parse_query_segments("users.name");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidQuery(msg) => {
                assert!(msg.contains("Missing parameter segment"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }

    #[test]
    fn test_parse_query_segments_multiple_segments() {
        // 修正: 複数セグメントは正常に処理される
        let result = parse_query_segments(".users.name.extra");
        assert!(result.is_ok()); 
        let (segment, fields) = result.unwrap();
        assert_eq!(segment, "users");
        assert_eq!(fields, vec!["name", "extra"]); // 3番目も含まれる
    }

    #[test]
    fn test_parse_query_segments_many_segments() {
        // さらに多くのセグメント
        let result = parse_query_segments(".a.b.c.d.e");
        assert!(result.is_ok());
        let (segment, fields) = result.unwrap();
        assert_eq!(segment, "a");
        assert_eq!(fields, vec!["b", "c", "d", "e"]);
    }

    #[test]
    fn test_parse_array_segment_normal_case() {
        // 正常ケース: 基本的な配列インデックス
        let result = parse_array_segment("users[0]");
        assert!(result.is_ok());
        let (idx, ridx) = result.unwrap();
        assert_eq!(idx, 5);  // '[' の位置
        assert_eq!(ridx, 7); // ']' の位置
    }

    #[test]
    fn test_parse_array_segment_large_index() {
        // 正常ケース: 大きなインデックス
        let result = parse_array_segment("items[123]");
        assert!(result.is_ok());
        let (idx, ridx) = result.unwrap();
        assert_eq!(idx, 5);   // '[' の位置
        assert_eq!(ridx, 9);  // ']' の位置
    }

    #[test]
    fn test_parse_array_segment_short_name() {
        // 正常ケース: 短いフィールド名
        let result = parse_array_segment("a[5]");
        assert!(result.is_ok());
        let (idx, ridx) = result.unwrap();
        assert_eq!(idx, 1);  // '[' の位置
        assert_eq!(ridx, 3); // ']' の位置
    }

    #[test]
    fn test_parse_array_segment_missing_open_bracket() {
        // エラーケース: '[' がない
        let result = parse_array_segment("users0]");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidQuery(msg) => {
                assert!(msg.contains("Missing '[' in segment"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }

    #[test]
    fn test_parse_array_segment_missing_close_bracket() {
        // エラーケース: ']' がない
        let result = parse_array_segment("users[0");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidQuery(msg) => {
                assert!(msg.contains("Missing ']' in segment"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }

    #[test]
    fn test_parse_array_segment_invalid_bracket_order() {
        // エラーケース: ブラケットの順序が逆
        let result = parse_array_segment("users]0[");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidQuery(msg) => {
                assert!(msg.contains("Invalid bracket order"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }

    #[test]
    fn test_parse_array_segment_empty_brackets() {
        // エラーケース: 空のブラケット
        let result = parse_array_segment("users[]");
        assert!(result.is_ok()); // パース自体は成功する
        let (idx, ridx) = result.unwrap();
        assert_eq!(idx, 5);  // '[' の位置
        assert_eq!(ridx, 6); // ']' の位置
    }

    #[test]
    fn test_parse_array_segment_no_brackets() {
        // エラーケース: ブラケットが全くない
        let result = parse_array_segment("users");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidQuery(msg) => {
                assert!(msg.contains("Missing '[' in segment"));
            }
            _ => panic!("Expected InvalidQuery error"),
        }
    }
}
