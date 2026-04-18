pub mod entity;
pub mod query;
pub mod world;

/// Converts a [`serde_json::Value`] to an equivalent Lua value.
pub(crate) fn json_to_lua(lua: &mlua::Lua, value: serde_json::Value) -> mlua::Result<mlua::Value> {
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
                table.raw_set(index + 1, json_to_lua(lua, element)?)?;
            }
            Ok(mlua::Value::Table(table))
        }
        Json::Object(object) => {
            let table = lua.create_table()?;
            for (key, val) in object {
                table.raw_set(key, json_to_lua(lua, val)?)?;
            }
            Ok(mlua::Value::Table(table))
        }
    }
}

/// Converts a Lua value to a [`serde_json::Value`].
pub(crate) fn lua_to_json(value: mlua::Value) -> mlua::Result<serde_json::Value> {
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
                let (key, val) = pair?;
                let key_string = match key {
                    mlua::Value::String(string) => string.to_str()?.to_owned(),
                    mlua::Value::Integer(integer) => integer.to_string(),
                    _ => continue,
                };
                object.insert(key_string, lua_to_json(val)?);
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
        let result =
            json_to_lua(&lua(), serde_json::Value::Null).expect("json_to_lua should succeed");
        assert_eq!(result, mlua::Value::Nil);
    }

    #[test]
    fn json_bool_true_becomes_lua_true() {
        let result =
            json_to_lua(&lua(), serde_json::json!(true)).expect("json_to_lua should succeed");
        assert_eq!(result, mlua::Value::Boolean(true));
    }

    #[test]
    fn json_bool_false_becomes_lua_false() {
        let result =
            json_to_lua(&lua(), serde_json::json!(false)).expect("json_to_lua should succeed");
        assert_eq!(result, mlua::Value::Boolean(false));
    }

    #[test]
    fn json_integer_becomes_lua_integer() {
        let result =
            json_to_lua(&lua(), serde_json::json!(42)).expect("json_to_lua should succeed");
        assert_eq!(result, mlua::Value::Integer(42));
    }

    #[test]
    fn json_float_becomes_lua_number() {
        let value = 1.5_f64;
        let result =
            json_to_lua(&lua(), serde_json::json!(value)).expect("json_to_lua should succeed");
        let mlua::Value::Number(number) = result else {
            panic!("expected Number")
        };
        assert!((number - value).abs() < f64::EPSILON);
    }

    #[test]
    fn json_string_becomes_lua_string() {
        let lua = lua();
        let result =
            json_to_lua(&lua, serde_json::json!("hello")).expect("json_to_lua should succeed");
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
        let lua = lua();
        let result =
            json_to_lua(&lua, serde_json::json!([10, 20, 30])).expect("json_to_lua should succeed");
        let mlua::Value::Table(table) = result else {
            panic!("expected Table")
        };
        assert_eq!(table.get::<i64>(1).expect("index 1 should exist"), 10);
        assert_eq!(table.get::<i64>(2).expect("index 2 should exist"), 20);
        assert_eq!(table.get::<i64>(3).expect("index 3 should exist"), 30);
    }

    #[test]
    fn json_object_becomes_lua_table_with_string_keys() {
        let lua = lua();
        let result = json_to_lua(&lua, serde_json::json!({ "x": 1, "y": 2 }))
            .expect("json_to_lua should succeed");
        let mlua::Value::Table(table) = result else {
            panic!("expected Table")
        };
        assert_eq!(table.get::<i64>("x").expect("key 'x' should exist"), 1);
        assert_eq!(table.get::<i64>("y").expect("key 'y' should exist"), 2);
    }

    #[test]
    fn lua_nil_becomes_json_null() {
        let result = lua_to_json(mlua::Value::Nil).expect("lua_to_json should succeed");
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn lua_bool_becomes_json_bool() {
        assert_eq!(
            lua_to_json(mlua::Value::Boolean(true)).expect("lua_to_json should succeed"),
            serde_json::json!(true)
        );

        assert_eq!(
            lua_to_json(mlua::Value::Boolean(false)).expect("lua_to_json should succeed"),
            serde_json::json!(false)
        );
    }

    #[test]
    fn lua_integer_becomes_json_number() {
        let result = lua_to_json(mlua::Value::Integer(99)).expect("lua_to_json should succeed");
        assert_eq!(result, serde_json::json!(99));
    }

    #[test]
    fn lua_float_becomes_json_number() {
        let result = lua_to_json(mlua::Value::Number(2.5)).expect("lua_to_json should succeed");
        assert!(
            (result.as_f64().expect("json value should be a float") - 2.5).abs() < f64::EPSILON
        );
    }

    #[test]
    fn lua_string_becomes_json_string() {
        let lua = lua();
        let string = lua
            .create_string("world")
            .expect("should create lua string");

        let result = lua_to_json(mlua::Value::String(string)).expect("lua_to_json should succeed");
        assert_eq!(result, serde_json::json!("world"));
    }

    #[test]
    fn lua_table_becomes_json_object() {
        let lua = lua();
        let table = lua.create_table().expect("should create lua table");
        table.set("hp", 50_i64).expect("table set should succeed");

        let result = lua_to_json(mlua::Value::Table(table)).expect("lua_to_json should succeed");
        assert_eq!(result["hp"], serde_json::json!(50));
    }

    #[test]
    fn lua_function_becomes_json_null() {
        let lua = lua();
        let function = lua
            .create_function(|_, ()| Ok(()))
            .expect("should create lua function");

        let result =
            lua_to_json(mlua::Value::Function(function)).expect("lua_to_json should succeed");
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn json_negative_integer_becomes_lua_integer() {
        let result =
            json_to_lua(&lua(), serde_json::json!(-99)).expect("json_to_lua should succeed");
        assert_eq!(result, mlua::Value::Integer(-99));
    }

    #[test]
    fn json_empty_array_becomes_empty_lua_table() {
        let lua = lua();
        let result = json_to_lua(&lua, serde_json::json!([])).expect("json_to_lua should succeed");
        let mlua::Value::Table(table) = result else {
            panic!("expected Table")
        };
        assert_eq!(table.len().expect("should get table length"), 0);
    }

    #[test]
    fn json_nested_object_roundtrip() {
        let lua = lua();
        let original = serde_json::json!({ "position": { "x": 1, "y": 2 } });
        let roundtrip =
            lua_to_json(json_to_lua(&lua, original).expect("json_to_lua should succeed"))
                .expect("lua_to_json should succeed");
        assert_eq!(roundtrip["position"]["x"], serde_json::json!(1));
        assert_eq!(roundtrip["position"]["y"], serde_json::json!(2));
    }

    #[test]
    fn lua_nan_float_becomes_zero_in_json() {
        let result =
            lua_to_json(mlua::Value::Number(f64::NAN)).expect("lua_to_json should succeed");
        assert_eq!(result, serde_json::json!(0));
    }

    #[test]
    fn integer_roundtrip() {
        let lua = lua();
        let original = serde_json::json!(123);
        let roundtrip =
            lua_to_json(json_to_lua(&lua, original.clone()).expect("json_to_lua should succeed"))
                .expect("lua_to_json should succeed");
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn object_roundtrip_preserves_keys_and_values() {
        let lua = lua();
        let original = serde_json::json!({ "name": "test", "value": 42 });
        let roundtrip =
            lua_to_json(json_to_lua(&lua, original.clone()).expect("json_to_lua should succeed"))
                .expect("lua_to_json should succeed");
        assert_eq!(roundtrip["name"], original["name"]);
        assert_eq!(roundtrip["value"], original["value"]);
    }
}
