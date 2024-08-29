use bevy::prelude::*;
use bevy::reflect::{DynamicList, DynamicMap, DynamicStruct, Map, Reflect};
use boa_engine::builtins::map::ordered_map::OrderedMap;
use boa_engine::builtins::set::ordered_set::OrderedSet;
use boa_engine::object::builtins::{JsArray, JsMap, JsSet};
use boa_engine::{js_str, Context, JsError, JsObject, JsResult, JsValue};

pub fn js_value_to_reflect(value: JsValue, ctx: &mut Context) -> JsResult<Box<dyn Reflect>> {
    match value {
        JsValue::Null | JsValue::Undefined => Ok(Box::new(()) as Box<dyn Reflect>),
        JsValue::Boolean(b) => Ok(Box::new(b)),
        JsValue::Integer(i) => Ok(Box::new(i as f32)),
        JsValue::Rational(f) => Ok(Box::new(f as f32)),
        JsValue::String(s) => Ok(Box::new(s.to_std_string_escaped())),
        JsValue::Object(obj) => {
            if obj.is_array() {
                return js_array_to_reflect(&JsArray::from_object(obj)?, ctx);
            }
            if obj.is::<OrderedMap<JsValue>>() {
                return js_map_to_reflect(&JsMap::from_object(obj)?, ctx);
            }
            if obj.is::<OrderedSet>() {
                return js_set_to_reflect(&JsSet::from_object(obj)?, ctx);
            }
            js_object_to_reflect(&obj, ctx)
        }
        JsValue::Symbol(_) => Err(JsError::from_opaque(
            js_str!("Symbol conversion not supported").into(),
        )),
        JsValue::BigInt(b) => Ok(Box::new(b.to_string())),
    }
}

fn js_array_to_reflect(array: &JsArray, ctx: &mut Context) -> JsResult<Box<dyn Reflect>> {
    let mut dynamic_list = DynamicList::default();
    for i in 0..array.length(ctx)? {
        let value = array.get(i, ctx)?;
        let reflect_value = js_value_to_reflect(value, ctx)?;
        dynamic_list.push_box(reflect_value);
    }
    Ok(Box::new(dynamic_list))
}

fn js_map_to_reflect(map: &JsMap, ctx: &mut Context) -> JsResult<Box<dyn Reflect>> {
    let mut dynamic_map = DynamicMap::default();
    let entries = map.entries(ctx)?;
    while let entry = entries.next(ctx)? {
        let entry = entry.to_object(ctx)?;
        let entry = JsArray::from_object(entry)?;

        let key = entry.get(0, ctx)?;
        let value = entry.get(1, ctx)?;
        let reflect_key = js_value_to_reflect(key, ctx)?;
        let reflect_value = js_value_to_reflect(value, ctx)?;
        dynamic_map.insert_boxed(reflect_key, reflect_value);
    }
    Ok(Box::new(dynamic_map))
}

fn js_set_to_reflect(set: &JsSet, ctx: &mut Context) -> JsResult<Box<dyn Reflect>> {
    let mut dynamic_list = DynamicList::default();
    let values = set.values(ctx)?;
    while let value = values.next(ctx)? {
        let reflect_value = js_value_to_reflect(value, ctx)?;
        dynamic_list.push_box(reflect_value);
    }
    Ok(Box::new(dynamic_list))
}

fn js_object_to_reflect(obj: &JsObject, ctx: &mut Context) -> JsResult<Box<dyn Reflect>> {
    let mut dynamic_struct = DynamicStruct::default();
    for key in obj.own_property_keys(ctx)? {
        let value = obj.get(key.clone(), ctx)?;
        let reflect_value = js_value_to_reflect(value, ctx)?;
        dynamic_struct.insert_boxed(key.to_string(), reflect_value);
    }

    if let Ok(variant) = obj.get(js_str!("__variant"), ctx) {
        if !variant.is_null_or_undefined() {
            // We can't handle enums right now... it's a bit complicated
            return Err(JsError::from_opaque(
                js_str!("Enums are not supported").into(),
            ));
        }
    }

    Ok(Box::new(dynamic_struct))
}
