//! Lua value conversion utilities shared across the `entity`, `query`, and `world` sub-modules.
//!
//! [`IntoLuaValueExt`] and [`IntoJsonValueExt`] bridge `serde_json::Value` ↔ `mlua::Value`.
//! [`IntoLuaValueExt::into_lua_table`] is the specialised form used by entity and query proxies
//! to convert a component snapshot directly into a Lua table.

pub mod entity;
pub mod query;
pub mod world;

/// Extension methods for converting JSON values into Lua values.
pub(crate) trait IntoLuaValueExt {
    /// Converts this JSON value into the equivalent Lua representation.
    fn into_lua_value(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value>;

    /// Converts this JSON value into a Lua table.
    ///
    /// Used by entity and query proxies to materialise component snapshots.
    /// If the JSON does not deserialise to a table (e.g. `null`) an empty
    /// table is returned so callers never need to handle that edge case.
    fn into_lua_table(self, lua: &mlua::Lua) -> mlua::Result<mlua::Table>;
}

impl IntoLuaValueExt for serde_json::Value {
    fn into_lua_value(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        use serde_json::Value as Json;
        match self {
            Json::Null => Ok(mlua::Value::Nil),
            Json::Bool(bool_value) => Ok(mlua::Value::Boolean(bool_value)),
            Json::Number(json_number) => {
                if let Some(integer_value) = json_number.as_i64() {
                    Ok(mlua::Value::Integer(integer_value))
                } else {
                    Ok(mlua::Value::Number(json_number.as_f64().unwrap_or(0.0)))
                }
            }
            Json::String(string_value) => {
                Ok(mlua::Value::String(lua.create_string(&string_value)?))
            }
            Json::Array(json_array) => {
                let lua_table = lua.create_table()?;
                for (array_index, array_value) in json_array.into_iter().enumerate() {
                    lua_table.raw_set(array_index + 1, array_value.into_lua_value(lua)?)?;
                }
                Ok(mlua::Value::Table(lua_table))
            }
            Json::Object(json_object) => {
                let lua_table = lua.create_table()?;
                for (object_key, object_value) in json_object {
                    lua_table.raw_set(object_key, object_value.into_lua_value(lua)?)?;
                }
                Ok(mlua::Value::Table(lua_table))
            }
        }
    }

    fn into_lua_table(self, lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
        match self.into_lua_value(lua)? {
            mlua::Value::Table(lua_table) => Ok(lua_table),
            _ => lua.create_table(),
        }
    }
}

/// Extension methods for converting Lua values into JSON values.
pub(crate) trait IntoJsonValueExt {
    /// Converts this Lua value into the equivalent JSON representation.
    fn into_json_value(self) -> mlua::Result<serde_json::Value>;
}

impl IntoJsonValueExt for mlua::Value {
    fn into_json_value(self) -> mlua::Result<serde_json::Value> {
        use serde_json::Value as Json;
        match self {
            mlua::Value::Nil => Ok(Json::Null),
            mlua::Value::Boolean(bool_value) => Ok(Json::Bool(bool_value)),
            mlua::Value::Integer(integer_value) => Ok(Json::Number(integer_value.into())),
            mlua::Value::Number(float_value) => {
                let json_number = serde_json::Number::from_f64(float_value)
                    .unwrap_or(serde_json::Number::from(0));
                Ok(Json::Number(json_number))
            }
            mlua::Value::String(lua_string) => Ok(Json::String(lua_string.to_str()?.to_owned())),
            mlua::Value::Table(lua_table) => {
                let mut json_object = serde_json::Map::new();
                for table_entry in lua_table.pairs::<mlua::Value, mlua::Value>() {
                    let (table_key, table_value) = table_entry?;
                    let json_key = match table_key {
                        mlua::Value::String(lua_string) => lua_string.to_str()?.to_owned(),
                        mlua::Value::Integer(integer_value) => integer_value.to_string(),
                        _ => continue,
                    };
                    json_object.insert(json_key, table_value.into_json_value()?);
                }
                Ok(Json::Object(json_object))
            }
            _ => Ok(Json::Null),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_null_becomes_lua_nil() {
        let lua = mlua::Lua::new();

        let result = serde_json::Value::Null
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        assert_eq!(result, mlua::Value::Nil);
    }

    #[test]
    fn json_bool_true_becomes_lua_true() {
        let lua = mlua::Lua::new();

        let result = serde_json::json!(true)
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        assert_eq!(result, mlua::Value::Boolean(true));
    }

    #[test]
    fn json_bool_false_becomes_lua_false() {
        let lua = mlua::Lua::new();

        let result = serde_json::json!(false)
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        assert_eq!(result, mlua::Value::Boolean(false));
    }

    #[test]
    fn json_integer_becomes_lua_integer() {
        let lua = mlua::Lua::new();

        let result = serde_json::json!(42)
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        assert_eq!(result, mlua::Value::Integer(42));
    }

    #[test]
    fn json_float_becomes_lua_number() {
        let lua = mlua::Lua::new();

        let value = 1.5_f64;
        let result = serde_json::json!(value)
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        let mlua::Value::Number(number) = result else {
            panic!("expected Number")
        };

        assert!((number - value).abs() < f64::EPSILON);
    }

    #[test]
    fn json_string_becomes_lua_string() {
        let lua = mlua::Lua::new();

        let result = serde_json::json!("hello")
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        let mlua::Value::String(string) = result else {
            panic!("expected String")
        };

        assert_eq!(
            string.to_str().expect("lua string should be valid utf-8"),
            "hello"
        );
    }

    #[test]
    fn json_array_becomes_one_based_lua_table() {
        let lua = mlua::Lua::new();

        let result = serde_json::json!([10, 20, 30])
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        let mlua::Value::Table(table) = result else {
            panic!("expected Table")
        };

        assert_eq!(table.get::<i64>(1).expect("index 1 should exist"), 10);

        assert_eq!(table.get::<i64>(2).expect("index 2 should exist"), 20);

        assert_eq!(table.get::<i64>(3).expect("index 3 should exist"), 30);
    }

    #[test]
    fn json_object_becomes_lua_table_with_string_keys() {
        let lua = mlua::Lua::new();

        let result = serde_json::json!({ "x": 1, "y": 2 })
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        let mlua::Value::Table(table) = result else {
            panic!("expected Table")
        };

        assert_eq!(table.get::<i64>("x").expect("key 'x' should exist"), 1);

        assert_eq!(table.get::<i64>("y").expect("key 'y' should exist"), 2);
    }

    #[test]
    fn lua_nil_becomes_json_null() {
        let result = mlua::Value::Nil
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn lua_bool_becomes_json_bool() {
        assert_eq!(
            mlua::Value::Boolean(true)
                .into_json_value()
                .expect("Lua to json conversion should succeed"),
            serde_json::json!(true)
        );

        assert_eq!(
            mlua::Value::Boolean(false)
                .into_json_value()
                .expect("Lua to json conversion should succeed"),
            serde_json::json!(false)
        );
    }

    #[test]
    fn lua_integer_becomes_json_number() {
        let result = mlua::Value::Integer(99)
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(result, serde_json::json!(99));
    }

    #[test]
    fn lua_float_becomes_json_number() {
        let result = mlua::Value::Number(2.5)
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert!(
            (result.as_f64().expect("json value should be a float") - 2.5).abs() < f64::EPSILON
        );
    }

    #[test]
    fn lua_string_becomes_json_string() {
        let lua = mlua::Lua::new();

        let string = lua
            .create_string("world")
            .expect("should create lua string");

        let result = mlua::Value::String(string)
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(result, serde_json::json!("world"));
    }

    #[test]
    fn lua_table_becomes_json_object() {
        let lua = mlua::Lua::new();

        let table = lua.create_table().expect("should create lua table");
        table.set("hp", 50_i64).expect("table set should succeed");

        let result = mlua::Value::Table(table)
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(result["hp"], serde_json::json!(50));
    }

    #[test]
    fn lua_function_becomes_json_null() {
        let lua = mlua::Lua::new();

        let function = lua
            .create_function(|_, ()| Ok(()))
            .expect("should create lua function");

        let result = mlua::Value::Function(function)
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn json_negative_integer_becomes_lua_integer() {
        let lua = mlua::Lua::new();

        let result = serde_json::json!(-99)
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        assert_eq!(result, mlua::Value::Integer(-99));
    }

    #[test]
    fn json_empty_array_becomes_empty_lua_table() {
        let lua = mlua::Lua::new();

        let result = serde_json::json!([])
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        let mlua::Value::Table(table) = result else {
            panic!("expected Table")
        };

        assert_eq!(table.len().expect("should get table length"), 0);
    }

    #[test]
    fn json_nested_object_roundtrip() {
        let lua = mlua::Lua::new();

        let original = serde_json::json!({ "position": { "x": 1, "y": 2 } });
        let roundtrip = original
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed")
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(roundtrip["position"]["x"], serde_json::json!(1));

        assert_eq!(roundtrip["position"]["y"], serde_json::json!(2));
    }

    #[test]
    fn lua_nan_float_becomes_zero_in_json() {
        let result = mlua::Value::Number(f64::NAN)
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(result, serde_json::json!(0));
    }

    #[test]
    fn integer_roundtrip() {
        let lua = mlua::Lua::new();

        let original = serde_json::json!(123);
        let roundtrip = original
            .clone()
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed")
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(roundtrip, original);
    }

    #[test]
    fn object_roundtrip_preserves_keys_and_values() {
        let lua = mlua::Lua::new();

        let original = serde_json::json!({ "name": "test", "value": 42 });
        let roundtrip = original
            .clone()
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed")
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(roundtrip["name"], original["name"]);

        assert_eq!(roundtrip["value"], original["value"]);
    }

    #[test]
    fn json_i64_max_roundtrips_through_lua() {
        let lua = mlua::Lua::new();

        let original = serde_json::json!(i64::MAX);
        let roundtrip = original
            .clone()
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed")
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(roundtrip, original);
    }

    #[test]
    fn json_i64_min_roundtrips_through_lua() {
        let lua = mlua::Lua::new();

        let original = serde_json::json!(i64::MIN);
        let roundtrip = original
            .clone()
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed")
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(roundtrip, original);
    }

    #[test]
    fn json_mixed_array_converts_each_element_type() {
        let lua = mlua::Lua::new();

        let original = serde_json::json!([1, "two", true, null]);
        let result = original
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        let mlua::Value::Table(table) = result else {
            panic!("expected Table");
        };

        assert_eq!(table.get::<i64>(1).expect("index 1 should be integer"), 1);

        assert_eq!(
            table.get::<String>(2).expect("index 2 should be string"),
            "two"
        );

        assert!(table.get::<bool>(3).expect("index 3 should be bool"));

        assert_eq!(
            table.get::<mlua::Value>(4).expect("index 4 should exist"),
            mlua::Value::Nil
        );
    }

    #[test]
    fn unicode_string_roundtrips() {
        let lua = mlua::Lua::new();

        let original = serde_json::json!("你好世界 🌍");
        let roundtrip = original
            .clone()
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed")
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(roundtrip, original);
    }

    #[test]
    fn json_empty_object_becomes_empty_lua_table() {
        let lua = mlua::Lua::new();

        let result = serde_json::json!({})
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        let mlua::Value::Table(table) = result else {
            panic!("expected Table");
        };

        assert_eq!(table.len().expect("should get table length"), 0);
    }

    #[test]
    fn lua_table_with_integer_key_appears_in_json_as_string_key() {
        let lua = mlua::Lua::new();

        let table = lua.create_table().expect("should create lua table");
        table.set(1_i64, "first").expect("table set should succeed");

        let result = mlua::Value::Table(table)
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert!(result.is_object());
        assert_eq!(result["1"], serde_json::json!("first"));
    }

    #[test]
    fn deeply_nested_object_roundtrips_correctly() {
        let lua = mlua::Lua::new();

        let original = serde_json::json!({
            "a": { "b": { "c": { "d": { "e": { "value": 42 } } } } }
        });

        let roundtrip = original
            .clone()
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed")
            .into_json_value()
            .expect("Lua to json conversion should succeed");

        assert_eq!(
            roundtrip["a"]["b"]["c"]["d"]["e"]["value"],
            serde_json::json!(42)
        );
    }

    #[test]
    fn json_array_with_nested_objects_roundtrips() {
        let lua = mlua::Lua::new();

        let original = serde_json::json!([{ "x": 1 }, { "x": 2 }]);
        let result = original
            .into_lua_value(&lua)
            .expect("json to Lua conversion should succeed");

        let mlua::Value::Table(table) = result else {
            panic!("expected Table");
        };

        let first: mlua::Table = table.get(1).expect("index 1 should exist");
        assert_eq!(first.get::<i64>("x").expect("x should exist"), 1);

        let second: mlua::Table = table.get(2).expect("index 2 should exist");
        assert_eq!(second.get::<i64>("x").expect("x should exist"), 2);
    }
}
