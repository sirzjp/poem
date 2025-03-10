use poem::{
    http::{StatusCode, Uri},
    Endpoint, IntoEndpoint, Request,
};
use poem_openapi::{
    param::Query,
    payload::Payload,
    registry::{MetaApi, MetaSchema, Registry},
    types::{multipart::JsonField, ParseFromJSON, Type},
    validation,
    validation::ValidatorMeta,
    Multipart, Object, OpenApi, OpenApiService,
};
use serde_json::json;

#[test]
fn test_u64() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        n: u64,
    }

    assert_eq!(A::parse_from_json(json!({ "n": 1 })).unwrap(), A { n: 1 });
}

#[test]
fn test_multiple_of() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(multiple_of = "10"))]
        n: i32,
    }

    assert_eq!(A::parse_from_json(json!({ "n": 20 })).unwrap(), A { n: 20 });
    assert_eq!(
        A::parse_from_json(json!({ "n": 25 }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `n` verification failed. multipleOf(10)"
    );

    let mut schema = MetaSchema::new("string");
    validation::MultipleOf::new(10.0).update_meta(&mut schema);
    assert_eq!(schema.multiple_of, Some(10.0));
}

#[test]
fn test_maximum() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(maximum(value = "500")))]
        n: i32,
    }

    assert_eq!(
        A::parse_from_json(json!({ "n": 400 })).unwrap(),
        A { n: 400 }
    );
    assert_eq!(
        A::parse_from_json(json!({ "n": 500 })).unwrap(),
        A { n: 500 }
    );
    assert_eq!(
        A::parse_from_json(json!({ "n": 530 }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `n` verification failed. maximum(500, exclusive: false)"
    );

    let mut schema = MetaSchema::new("string");
    validation::Maximum::new(10.0, false).update_meta(&mut schema);
    assert_eq!(schema.maximum, Some(10.0));
    assert_eq!(schema.exclusive_maximum, None);
}

#[test]
fn test_maximum_exclusive() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(maximum(value = "500", exclusive)))]
        n: i32,
    }

    assert_eq!(
        A::parse_from_json(json!({ "n": 400 })).unwrap(),
        A { n: 400 }
    );
    assert_eq!(
        A::parse_from_json(json!({ "n": 500 }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `n` verification failed. maximum(500, exclusive: true)"
    );
    assert_eq!(
        A::parse_from_json(json!({ "n": 530 }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `n` verification failed. maximum(500, exclusive: true)"
    );

    let mut schema = MetaSchema::new("string");
    validation::Maximum::new(10.0, true).update_meta(&mut schema);
    assert_eq!(schema.maximum, Some(10.0));
    assert_eq!(schema.exclusive_maximum, Some(true));
}

#[test]
fn test_max_length() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(max_length = "5"))]
        value: String,
    }

    assert_eq!(
        A::parse_from_json(json!({ "value": "abcd" })).unwrap(),
        A {
            value: "abcd".to_string()
        }
    );
    assert_eq!(
        A::parse_from_json(json!({ "value": "abcdef" }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `value` verification failed. maxLength(5)"
    );

    let mut schema = MetaSchema::new("string");
    validation::MaxLength::new(10).update_meta(&mut schema);
    assert_eq!(schema.max_length, Some(10));
}

#[test]
fn test_min_length() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(min_length = "5"))]
        value: String,
    }

    assert_eq!(
        A::parse_from_json(json!({ "value": "abcdef" })).unwrap(),
        A {
            value: "abcdef".to_string()
        }
    );
    assert_eq!(
        A::parse_from_json(json!({ "value": "abcd" }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `value` verification failed. minLength(5)"
    );

    let mut schema = MetaSchema::new("string");
    validation::MinLength::new(10).update_meta(&mut schema);
    assert_eq!(schema.min_length, Some(10));
}

#[test]
fn test_pattern() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(pattern = r#"\[.*\]"#))]
        value: String,
    }

    assert_eq!(
        A::parse_from_json(json!({ "value": "[123]" })).unwrap(),
        A {
            value: "[123]".to_string()
        }
    );
    assert_eq!(
        A::parse_from_json(json!({ "value": "123" }))
            .unwrap_err()
            .into_message(),
        r#"failed to parse "A": field `value` verification failed. pattern("\[.*\]")"#
    );

    let mut schema = MetaSchema::new("string");
    validation::Pattern::new(r#"\[.*\]"#).update_meta(&mut schema);
    assert_eq!(schema.pattern.as_deref(), Some(r#"\[.*\]"#));
}

#[test]
fn test_max_items() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(max_items = "3"))]
        values: Vec<String>,
    }

    assert_eq!(
        A::parse_from_json(json!({ "values": ["1", "2", "3"] })).unwrap(),
        A {
            values: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        }
    );
    assert_eq!(
        A::parse_from_json(json!({ "values": ["1", "2", "3", "4"] }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `values` verification failed. maxItems(3)"
    );

    let mut schema = MetaSchema::new("string");
    validation::MaxItems::new(10).update_meta(&mut schema);
    assert_eq!(schema.max_items, Some(10));
}

#[test]
fn test_min_items() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(min_items = "4"))]
        values: Vec<String>,
    }

    assert_eq!(
        A::parse_from_json(json!({ "values": ["1", "2", "3", "4"] })).unwrap(),
        A {
            values: vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string()
            ],
        }
    );
    assert_eq!(
        A::parse_from_json(json!({ "values": ["1", "2", "3"] }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `values` verification failed. minItems(4)"
    );

    let mut schema = MetaSchema::new("string");
    validation::MinItems::new(10).update_meta(&mut schema);
    assert_eq!(schema.min_items, Some(10));
}

#[test]
fn test_unique_items() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(unique_items))]
        values: Vec<String>,
    }

    assert_eq!(
        A::parse_from_json(json!({ "values": ["1", "2", "3"] })).unwrap(),
        A {
            values: vec!["1".to_string(), "2".to_string(), "3".to_string(),],
        }
    );
    assert_eq!(
        A::parse_from_json(json!({ "values": ["1", "2", "2"] }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `values` verification failed. uniqueItems()"
    );

    let mut schema = MetaSchema::new("string");
    validation::UniqueItems.update_meta(&mut schema);
    assert_eq!(schema.unique_items, Some(true));
}

#[tokio::test]
async fn param_validator() {
    struct Api;

    #[OpenApi]
    impl Api {
        #[oai(path = "/", method = "get")]
        async fn test(
            &self,
            #[oai(name = "v", validator(maximum(value = "100", exclusive)))] _v: Query<i32>,
        ) {
        }
    }

    let api = OpenApiService::new(Api, "test", "1.0").into_endpoint();
    let mut resp = api
        .call(Request::builder().uri(Uri::from_static("/?v=999")).finish())
        .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.take_body().into_string().await.unwrap(),
        "Failed to parse parameter `v`: verification failed. maximum(100, exclusive: true)"
    );

    let meta: MetaApi = Api::meta().remove(0);
    assert_eq!(
        meta.paths[0].operations[0].params[0]
            .schema
            .unwrap_inline()
            .maximum,
        Some(100.0)
    );
    assert_eq!(
        meta.paths[0].operations[0].params[0]
            .schema
            .unwrap_inline()
            .exclusive_maximum,
        Some(true)
    );

    let resp = api
        .call(Request::builder().uri(Uri::from_static("/?v=50")).finish())
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[test]
fn test_option() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(multiple_of = "10"))]
        n1: Option<i32>,
        #[oai(validator(multiple_of = "10"))]
        n2: Option<Option<i32>>,
    }

    assert_eq!(
        A::parse_from_json(json!({ "n1": 20 })).unwrap(),
        A {
            n1: Some(20),
            n2: None
        }
    );

    assert_eq!(
        A::parse_from_json(json!({ "n2": 20 })).unwrap(),
        A {
            n1: None,
            n2: Some(Some(20)),
        }
    );

    assert_eq!(
        A::parse_from_json(json!({ "n2": null })).unwrap(),
        A { n1: None, n2: None }
    );

    assert_eq!(
        A::parse_from_json(json!({ "n1": 25 }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `n1` verification failed. multipleOf(10)"
    );

    assert_eq!(
        A::parse_from_json(json!({ "n2": 25 }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `n2` verification failed. multipleOf(10)"
    );
}

#[test]
fn test_multiple_validators() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(multiple_of = "10", maximum(value = "500")))]
        n: i32,
    }

    assert_eq!(A::parse_from_json(json!({ "n": 20 })).unwrap(), A { n: 20 });
    assert_eq!(
        A::parse_from_json(json!({ "n": 25 }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `n` verification failed. multipleOf(10)"
    );

    assert_eq!(
        A::parse_from_json(json!({ "n": 530 }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `n` verification failed. maximum(500, exclusive: false)"
    );
}

#[test]
fn test_unsigned_integers() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
    }
    assert_eq!(
        A::parse_from_json(json!({
            "u8": u8::MAX as u64,
            "u16": u16::MAX as u64,
            "u32": u32::MAX as u64,
            "u64": u64::MAX as u64,
        }))
        .unwrap(),
        A {
            u8: u8::MAX,
            u16: u16::MAX,
            u32: u32::MAX,
            u64: u64::MAX,
        }
    );
    assert_eq!(
        A::parse_from_json(json!({
            "u8": u8::MAX as u64 + 1,
            "u16": u16::MAX as u64,
            "u32": u32::MAX as u64,
            "u64": u64::MAX as u64,
        }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"integer(uint8)\": Only integers from 0 to 255 are accepted. (occurred while parsing \"A\")"
    );
}

#[test]
fn test_list_on_object() {
    #[derive(Object, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(list, maximum(value = "10")))]
        n: Vec<i32>,
    }

    assert_eq!(
        A::parse_from_json(json!({ "n": [1, 2, 3] })).unwrap(),
        A { n: vec![1, 2, 3] }
    );
    assert_eq!(
        A::parse_from_json(json!({ "n": [1, 2, 3, 25] }))
            .unwrap_err()
            .into_message(),
        "failed to parse \"A\": field `n` verification failed. maximum(10, exclusive: false)"
    );

    let mut registry = Registry::default();
    A::register(&mut registry);
    let schema = registry.schemas.get_mut("A").unwrap();
    let (name, field_n) = schema.properties.remove(0);
    assert_eq!(name, "n");

    let schema_n = field_n.unwrap_inline();
    let schema_items = schema_n.items.as_ref().unwrap();
    let schema_items = schema_items.unwrap_inline();
    assert_eq!(schema_items.maximum, Some(10.0));
}

#[test]
fn test_list_on_multipart() {
    #[derive(Multipart, Debug, Eq, PartialEq)]
    struct A {
        #[oai(validator(list, maximum(value = "32")))]
        values: Vec<JsonField<i32>>,
    }

    let schema_ref = A::schema_ref();
    let schema: &MetaSchema = schema_ref.unwrap_inline();
    assert_eq!(schema.ty, "object");
    assert_eq!(schema.properties.len(), 1);

    assert_eq!(schema.properties[0].0, "values");
    let schema_values = schema.properties[0].1.unwrap_inline();
    assert_eq!(schema_values.ty, "array");

    let schema_items = schema_values.items.as_ref().unwrap();
    let schema_items = schema_items.unwrap_inline();
    assert_eq!(schema_items.maximum, Some(32.0));
}
