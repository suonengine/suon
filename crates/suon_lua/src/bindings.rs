use mlua::{Error, Lua, Table, Value};
use suon_adler32;

fn json_to_lua(lua: &Lua, value: serde_json::Value) -> Result<Value, Error> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::String(lua.create_string(n.to_string())?))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(&s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.into_iter().enumerate() {
                table.raw_set(i + 1, json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(map) => {
            let table = lua.create_table()?;
            for (k, v) in map {
                table.raw_set(k, json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

fn lua_to_json(value: Value) -> Result<serde_json::Value, Error> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        Value::Integer(i) => Ok(serde_json::Value::Number(i.into())),
        Value::Number(n) => {
            let json_num = serde_json::Number::from_f64(n)
                .ok_or_else(|| Error::external(format!("NaN/Infinity not allowed in JSON: {n}")))?;
            Ok(serde_json::Value::Number(json_num))
        }
        Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_owned())),
        Value::Table(table) => json_from_table(table),
        _ => Err(Error::external("unsupported Lua type for JSON conversion")),
    }
}

fn json_from_table(table: Table) -> Result<serde_json::Value, Error> {
    let mut pairs = Vec::new();

    for pair in table.pairs::<Value, Value>() {
        let (k, v) = pair?;
        let key = match k {
            Value::String(s) => s.to_str()?.to_owned(),
            Value::Integer(i) => i.to_string(),
            other => format!("{other:?}"),
        };
        pairs.push((key, lua_to_json(v)?));
    }

    if pairs.is_empty() {
        return Ok(serde_json::Value::Object(serde_json::Map::new()));
    }

    let is_array = pairs
        .iter()
        .enumerate()
        .all(|(i, (key, _))| key.parse::<usize>().map_or(false, |n| n == i + 1));

    if is_array {
        let arr = pairs.into_iter().map(|(_, v)| v).collect();
        return Ok(serde_json::Value::Array(arr));
    }

    let mut map = serde_json::Map::new();
    for (k, v) in pairs {
        map.insert(k, v);
    }

    Ok(serde_json::Value::Object(map))
}

fn http_post(url: String, body: String) -> Result<(String, i64), ureq::Error> {
    let response = ureq::post(&url)
        .header("Content-Type", "application/json")
        .send(body)?;
    let status = response.status().as_u16() as i64;
    let text = response.into_body().read_to_string().unwrap_or_default();
    Ok((text, status))
}

fn http_get(url: String) -> Result<(String, i64), ureq::Error> {
    let response = ureq::get(&url).call()?;
    let status = response.status().as_u16() as i64;
    let text = response.into_body().read_to_string().unwrap_or_default();
    Ok((text, status))
}

fn handle_http_result(
    result: Result<Result<(String, i64), ureq::Error>, tokio::task::JoinError>,
) -> Result<(String, i64), Error> {
    match result {
        Ok(Ok((text, status))) => Ok((text, status)),
        Ok(Err(ureq::Error::StatusCode(code))) => Ok((String::new(), code as i64)),
        Ok(Err(err)) => Err(Error::external(format!("HTTP request failed: {err}"))),
        Err(join_err) => Err(Error::external(format!("HTTP request failed: {join_err}"))),
    }
}

pub fn inject_bindings(lua: &Lua) -> Result<(), Error> {
    let globals = lua.globals();

    globals.set(
        "adler32",
        lua.create_function(|_, data: mlua::String| {
            let bytes = data.as_bytes();
            Ok(suon_adler32::generate(&bytes))
        })?,
    )?;

    let json = lua.create_table()?;
    json.set(
        "decode",
        lua.create_function(|lua, text: String| {
            let value = serde_json::from_str(&text)
                .map_err(|e| Error::external(format!("JSON decode error: {e}")))?;
            json_to_lua(lua, value)
        })?,
    )?;
    json.set(
        "encode",
        lua.create_function(|_, value: Value| {
            let json_value = lua_to_json(value)?;
            serde_json::to_string(&json_value)
                .map_err(|e| Error::external(format!("JSON encode error: {e}")))
        })?,
    )?;
    globals.set("Json", json)?;

    let http = lua.create_table()?;
    http.set(
        "post",
        lua.create_function(|_, (url, body): (String, String)| {
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(tokio::task::spawn_blocking(move || http_post(url, body)))
            });
            handle_http_result(result)
        })?,
    )?;
    http.set(
        "get",
        lua.create_function(|_, url: String| {
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(tokio::task::spawn_blocking(move || http_get(url)))
            });
            handle_http_result(result)
        })?,
    )?;
    globals.set("Http", http)?;

    Ok(())
}
