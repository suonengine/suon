//! Lua value conversion helpers and sub-modules for the world/entity/query APIs.
//!
//! [`json_value_to_lua_value`] and [`lua_value_to_json_value`] are the serialisation bridge used by every
//! component accessor: ECS components travel as `serde_json::Value` between Rust and Lua.

pub mod entity;
pub mod query;
pub mod world;

/// Converts a [`serde_json::Value`] to an equivalent Lua value.
pub(crate) fn json_value_to_lua_value(
    lua: &mlua::Lua,
    value: serde_json::Value,
) -> mlua::Result<mlua::Value> {
    use serde_json::Value as Json;
    match value {
        Json::Null => Ok(mlua::Value::Nil),
        Json::Bool(boolean) => Ok(mlua::Value::Boolean(boolean)),
        Json::Number(number) => {
            if let Some(integer) = number.as_i64() {
                Ok(mlua::Value::Integer(integer))
            } else {
                Ok(mlua::Value::Number(number.as_f64().unwrap_or(0.0)))
            }
        }
        Json::String(string) => Ok(mlua::Value::String(lua.create_string(&string)?)),
        Json::Array(array) => {
            let table = lua.create_table()?;
            for (index, element) in array.into_iter().enumerate() {
                table.raw_set(index + 1, json_value_to_lua_value(lua, element)?)?;
            }
            Ok(mlua::Value::Table(table))
        }
        Json::Object(object) => {
            let table = lua.create_table()?;
            for (key, value) in object {
                table.raw_set(key, json_value_to_lua_value(lua, value)?)?;
            }
            Ok(mlua::Value::Table(table))
        }
    }
}

/// Converts a Lua value to a [`serde_json::Value`].
pub(crate) fn lua_value_to_json_value(value: mlua::Value) -> mlua::Result<serde_json::Value> {
    use serde_json::Value as Json;
    match value {
        mlua::Value::Nil => Ok(Json::Null),
        mlua::Value::Boolean(boolean) => Ok(Json::Bool(boolean)),
        mlua::Value::Integer(integer) => Ok(Json::Number(integer.into())),
        mlua::Value::Number(float) => {
            let number = serde_json::Number::from_f64(float).unwrap_or(serde_json::Number::from(0));
            Ok(Json::Number(number))
        }
        mlua::Value::String(string) => Ok(Json::String(string.to_str()?.to_owned())),
        mlua::Value::Table(table) => {
            let mut object = serde_json::Map::new();
            for pair in table.pairs::<mlua::Value, mlua::Value>() {
                let (key, value) = pair?;
                let key_string = match key {
                    mlua::Value::String(string) => string.to_str()?.to_owned(),
                    mlua::Value::Integer(integer) => integer.to_string(),
                    _ => continue,
                };
                object.insert(key_string, lua_value_to_json_value(value)?);
            }
            Ok(Json::Object(object))
        }
        _ => Ok(Json::Null),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lua() -> mlua::Lua {
        mlua::Lua::new()
    }

    #[test]
    fn json_null_becomes_lua_nil() {
        let result = json_value_to_lua_value(&lua(), serde_json::Value::Null)
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        assert_eq!(result, mlua::Value::Nil);
    }

    #[test]
    fn json_bool_true_becomes_lua_true() {
        let result = json_value_to_lua_value(&lua(), serde_json::json!(true))
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        assert_eq!(result, mlua::Value::Boolean(true));
    }

    #[test]
    fn json_bool_false_becomes_lua_false() {
        let result = json_value_to_lua_value(&lua(), serde_json::json!(false))
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        assert_eq!(result, mlua::Value::Boolean(false));
    }

    #[test]
    fn json_integer_becomes_lua_integer() {
        let result = json_value_to_lua_value(&lua(), serde_json::json!(42))
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        assert_eq!(result, mlua::Value::Integer(42));
    }

    #[test]
    fn json_float_becomes_lua_number() {
        let value = 1.5_f64;
        let result = json_value_to_lua_value(&lua(), serde_json::json!(value))
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        let mlua::Value::Number(number) = result else {
            panic!("expected Number")
        };
        assert!((number - value).abs() < f64::EPSILON);
    }

    #[test]
    fn json_string_becomes_lua_string() {
        let lua = lua();
        let result = json_value_to_lua_value(&lua, serde_json::json!("hello"))
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        let mlua::Value::String(string) = result else {
            panic!("expected String")
        };
        assert_eq!(
            string
                .to_str()
                .unwrap_or_else(|error| panic!("lua string should be valid utf-8: {error}")),
            "hello"
        );
    }

    #[test]
    fn json_array_becomes_one_based_lua_table() {
        let lua = lua();
        let result = json_value_to_lua_value(&lua, serde_json::json!([10, 20, 30]))
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        let mlua::Value::Table(table) = result else {
            panic!("expected Table")
        };
        assert_eq!(
            table
                .get::<i64>(1)
                .unwrap_or_else(|error| panic!("index 1 should exist: {error}")),
            10
        );
        assert_eq!(
            table
                .get::<i64>(2)
                .unwrap_or_else(|error| panic!("index 2 should exist: {error}")),
            20
        );
        assert_eq!(
            table
                .get::<i64>(3)
                .unwrap_or_else(|error| panic!("index 3 should exist: {error}")),
            30
        );
    }

    #[test]
    fn json_object_becomes_lua_table_with_string_keys() {
        let lua = lua();
        let result = json_value_to_lua_value(&lua, serde_json::json!({ "x": 1, "y": 2 }))
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        let mlua::Value::Table(table) = result else {
            panic!("expected Table")
        };
        assert_eq!(
            table
                .get::<i64>("x")
                .unwrap_or_else(|error| panic!("key 'x' should exist: {error}")),
            1
        );
        assert_eq!(
            table
                .get::<i64>("y")
                .unwrap_or_else(|error| panic!("key 'y' should exist: {error}")),
            2
        );
    }

    #[test]
    fn lua_nil_becomes_json_null() {
        let result = lua_value_to_json_value(mlua::Value::Nil)
            .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn lua_bool_becomes_json_bool() {
        assert_eq!(
            lua_value_to_json_value(mlua::Value::Boolean(true))
                .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}")),
            serde_json::json!(true)
        );

        assert_eq!(
            lua_value_to_json_value(mlua::Value::Boolean(false))
                .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}")),
            serde_json::json!(false)
        );
    }

    #[test]
    fn lua_integer_becomes_json_number() {
        let result = lua_value_to_json_value(mlua::Value::Integer(99))
            .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(result, serde_json::json!(99));
    }

    #[test]
    fn lua_float_becomes_json_number() {
        let result = lua_value_to_json_value(mlua::Value::Number(2.5))
            .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert!(
            (result
                .as_f64()
                .unwrap_or_else(|| panic!("json value should be a float"))
                - 2.5)
                .abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn lua_string_becomes_json_string() {
        let lua = lua();
        let string = lua
            .create_string("world")
            .unwrap_or_else(|error| panic!("should create lua string: {error}"));
        let result = lua_value_to_json_value(mlua::Value::String(string))
            .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(result, serde_json::json!("world"));
    }

    #[test]
    fn lua_table_becomes_json_object() {
        let lua = lua();
        let table = lua
            .create_table()
            .unwrap_or_else(|error| panic!("should create lua table: {error}"));
        table
            .set("hp", 50_i64)
            .unwrap_or_else(|error| panic!("table set should succeed: {error}"));
        let result = lua_value_to_json_value(mlua::Value::Table(table))
            .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(result["hp"], serde_json::json!(50));
    }

    #[test]
    fn lua_function_becomes_json_null() {
        let lua = lua();
        let function = lua
            .create_function(|_, ()| Ok(()))
            .unwrap_or_else(|error| panic!("should create lua function: {error}"));
        let result = lua_value_to_json_value(mlua::Value::Function(function))
            .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn json_negative_integer_becomes_lua_integer() {
        let result = json_value_to_lua_value(&lua(), serde_json::json!(-99))
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        assert_eq!(result, mlua::Value::Integer(-99));
    }

    #[test]
    fn json_empty_array_becomes_empty_lua_table() {
        let lua = lua();
        let result = json_value_to_lua_value(&lua, serde_json::json!([]))
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        let mlua::Value::Table(table) = result else {
            panic!("expected Table")
        };
        assert_eq!(
            table
                .len()
                .unwrap_or_else(|error| panic!("should get table length: {error}")),
            0
        );
    }

    #[test]
    fn json_nested_object_roundtrip() {
        let lua = lua();
        let original = serde_json::json!({ "position": { "x": 1, "y": 2 } });
        let roundtrip = lua_value_to_json_value(
            json_value_to_lua_value(&lua, original)
                .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}")),
        )
        .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(roundtrip["position"]["x"], serde_json::json!(1));
        assert_eq!(roundtrip["position"]["y"], serde_json::json!(2));
    }

    #[test]
    fn lua_nan_float_becomes_zero_in_json() {
        let result = lua_value_to_json_value(mlua::Value::Number(f64::NAN))
            .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(result, serde_json::json!(0));
    }

    #[test]
    fn integer_roundtrip() {
        let lua = lua();
        let original = serde_json::json!(123);
        let roundtrip = lua_value_to_json_value(
            json_value_to_lua_value(&lua, original.clone())
                .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}")),
        )
        .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn object_roundtrip_preserves_keys_and_values() {
        let lua = lua();
        let original = serde_json::json!({ "name": "test", "value": 42 });
        let roundtrip = lua_value_to_json_value(
            json_value_to_lua_value(&lua, original.clone())
                .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}")),
        )
        .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(roundtrip["name"], original["name"]);
        assert_eq!(roundtrip["value"], original["value"]);
    }

    #[test]
    fn json_i64_max_roundtrips_through_lua() {
        let lua = lua();
        let original = serde_json::json!(i64::MAX);
        let roundtrip = lua_value_to_json_value(
            json_value_to_lua_value(&lua, original.clone())
                .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}")),
        )
        .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn json_i64_min_roundtrips_through_lua() {
        let lua = lua();
        let original = serde_json::json!(i64::MIN);
        let roundtrip = lua_value_to_json_value(
            json_value_to_lua_value(&lua, original.clone())
                .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}")),
        )
        .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn json_mixed_array_converts_each_element_type() {
        let lua = lua();
        let original = serde_json::json!([1, "two", true, null]);
        let result = json_value_to_lua_value(&lua, original)
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        let mlua::Value::Table(table) = result else {
            panic!("expected Table");
        };
        assert_eq!(
            table
                .get::<i64>(1)
                .unwrap_or_else(|error| panic!("index 1 should be integer: {error}")),
            1
        );
        assert_eq!(
            table
                .get::<String>(2)
                .unwrap_or_else(|error| panic!("index 2 should be string: {error}")),
            "two"
        );
        assert!(
            table
                .get::<bool>(3)
                .unwrap_or_else(|error| panic!("index 3 should be bool: {error}"))
        );
        assert_eq!(
            table
                .get::<mlua::Value>(4)
                .unwrap_or_else(|error| panic!("index 4 should exist: {error}")),
            mlua::Value::Nil
        );
    }

    #[test]
    fn unicode_string_roundtrips() {
        let lua = lua();
        let original = serde_json::json!("你好世界 🌍");
        let roundtrip = lua_value_to_json_value(
            json_value_to_lua_value(&lua, original.clone())
                .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}")),
        )
        .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn json_empty_object_becomes_empty_lua_table() {
        let lua = lua();
        let result = json_value_to_lua_value(&lua, serde_json::json!({}))
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        let mlua::Value::Table(table) = result else {
            panic!("expected Table");
        };
        assert_eq!(
            table
                .len()
                .unwrap_or_else(|error| panic!("should get table length: {error}")),
            0
        );
    }

    #[test]
    fn lua_table_with_integer_key_appears_in_json_as_string_key() {
        let lua = lua();
        let table = lua
            .create_table()
            .unwrap_or_else(|error| panic!("should create lua table: {error}"));
        table
            .set(1_i64, "first")
            .unwrap_or_else(|error| panic!("table set should succeed: {error}"));
        let result = lua_value_to_json_value(mlua::Value::Table(table))
            .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert!(result.is_object());
        assert_eq!(result["1"], serde_json::json!("first"));
    }

    #[test]
    fn deeply_nested_object_roundtrips_correctly() {
        let lua = lua();
        let original = serde_json::json!({
            "a": { "b": { "c": { "d": { "e": { "value": 42 } } } } }
        });
        let roundtrip = lua_value_to_json_value(
            json_value_to_lua_value(&lua, original.clone())
                .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}")),
        )
        .unwrap_or_else(|error| panic!("lua_value_to_json_value should succeed: {error}"));
        assert_eq!(
            roundtrip["a"]["b"]["c"]["d"]["e"]["value"],
            serde_json::json!(42)
        );
    }

    #[test]
    fn json_array_with_nested_objects_roundtrips() {
        let lua = lua();
        let original = serde_json::json!([{ "x": 1 }, { "x": 2 }]);
        let result = json_value_to_lua_value(&lua, original)
            .unwrap_or_else(|error| panic!("json_value_to_lua_value should succeed: {error}"));
        let mlua::Value::Table(table) = result else {
            panic!("expected Table");
        };
        let first: mlua::Table = table
            .get(1)
            .unwrap_or_else(|error| panic!("index 1 should exist: {error}"));
        let second: mlua::Table = table
            .get(2)
            .unwrap_or_else(|error| panic!("index 2 should exist: {error}"));
        assert_eq!(
            first
                .get::<i64>("x")
                .unwrap_or_else(|error| panic!("x should exist: {error}")),
            1
        );
        assert_eq!(
            second
                .get::<i64>("x")
                .unwrap_or_else(|error| panic!("x should exist: {error}")),
            2
        );
    }
}
