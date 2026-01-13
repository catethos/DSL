//! Value conversion between LatticeValue and JavaScript values

use neon::prelude::*;

use lattice::runtime::LatticeValue;

/// Convert LatticeValue to JavaScript value
pub fn lattice_value_to_js<'a>(
    cx: &mut impl Context<'a>,
    value: &LatticeValue,
) -> JsResult<'a, JsValue> {
    match value {
        LatticeValue::Null => Ok(cx.null().upcast()),

        LatticeValue::Bool(b) => Ok(cx.boolean(*b).upcast()),

        LatticeValue::Int(i) => {
            // JavaScript numbers are f64, safe for i53 integers
            Ok(cx.number(*i as f64).upcast())
        }

        LatticeValue::Float(f) => Ok(cx.number(*f).upcast()),

        LatticeValue::String(s) => Ok(cx.string(s).upcast()),

        LatticeValue::Path(p) => {
            // Return as { __lattice_path__: true, value: "path/string" }
            let obj = cx.empty_object();
            let marker = cx.boolean(true);
            obj.set(cx, "__lattice_path__", marker)?;
            let value = cx.string(p);
            obj.set(cx, "value", value)?;
            Ok(obj.upcast())
        }

        LatticeValue::List(items) => {
            let arr = cx.empty_array();
            for (i, item) in items.iter().enumerate() {
                let js_item = lattice_value_to_js(cx, item)?;
                arr.set(cx, i as u32, js_item)?;
            }
            Ok(arr.upcast())
        }

        LatticeValue::Map(pairs) => {
            let obj = cx.empty_object();
            for (key, value) in pairs {
                let js_value = lattice_value_to_js(cx, value)?;
                obj.set(cx, key.as_str(), js_value)?;
            }
            Ok(obj.upcast())
        }
    }
}

/// Convert JavaScript value to LatticeValue
pub fn js_to_lattice_value<'a>(
    cx: &mut impl Context<'a>,
    value: Handle<'a, JsValue>,
) -> NeonResult<LatticeValue> {
    // Check for null/undefined
    if value.is_a::<JsNull, _>(cx) || value.is_a::<JsUndefined, _>(cx) {
        return Ok(LatticeValue::Null);
    }

    // Check for boolean
    if let Ok(b) = value.downcast::<JsBoolean, _>(cx) {
        return Ok(LatticeValue::Bool(b.value(cx)));
    }

    // Check for number
    if let Ok(n) = value.downcast::<JsNumber, _>(cx) {
        let f = n.value(cx);
        // Check if it's an integer (whole number within i64 range)
        if f.trunc() == f && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
            return Ok(LatticeValue::Int(f as i64));
        }
        return Ok(LatticeValue::Float(f));
    }

    // Check for string
    if let Ok(s) = value.downcast::<JsString, _>(cx) {
        return Ok(LatticeValue::String(s.value(cx)));
    }

    // Check for array
    if let Ok(arr) = value.downcast::<JsArray, _>(cx) {
        let len = arr.len(cx);
        let mut items = Vec::with_capacity(len as usize);
        for i in 0..len {
            let item: Handle<JsValue> = arr.get(cx, i)?;
            items.push(js_to_lattice_value(cx, item)?);
        }
        return Ok(LatticeValue::List(items));
    }

    // Check for object (including Path marker)
    if let Ok(obj) = value.downcast::<JsObject, _>(cx) {
        // Check for Path marker
        if let Ok(marker) = obj.get::<JsBoolean, _, _>(cx, "__lattice_path__") {
            if marker.value(cx) {
                let path_value: Handle<JsString> = obj.get(cx, "value")?;
                return Ok(LatticeValue::Path(path_value.value(cx)));
            }
        }

        // Regular object -> Map
        let names = obj.get_own_property_names(cx)?;
        let len = names.len(cx);
        let mut pairs = Vec::with_capacity(len as usize);

        for i in 0..len {
            let key: Handle<JsString> = names.get(cx, i)?;
            let key_str = key.value(cx);
            let val: Handle<JsValue> = obj.get(cx, key_str.as_str())?;
            pairs.push((key_str, js_to_lattice_value(cx, val)?));
        }

        return Ok(LatticeValue::Map(pairs));
    }

    cx.throw_type_error("Cannot convert value to LatticeValue")
}
