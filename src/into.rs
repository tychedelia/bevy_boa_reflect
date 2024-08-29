use anyhow::Context as AnyhowContext;
use bevy::prelude::*;
use bevy::reflect::{Array, Enum, List, Map, Reflect, ReflectRef, Tuple};
use boa_engine::object::builtins::{JsArray, JsMap, JsSet};
use boa_engine::property::Attribute;
use boa_engine::{
    js_str, object::ObjectInitializer, Context, JsError, JsResult, JsString, JsValue,
};

pub fn reflect_to_js_value(value: &dyn Reflect, ctx: &mut Context) -> JsResult<JsValue> {
    match value.reflect_ref() {
        ReflectRef::Struct(s) => reflect_to_js_object(s, ctx),
        ReflectRef::TupleStruct(t) => reflect_tuple_struct_to_js_array(t, ctx),
        ReflectRef::Tuple(t) => reflect_tuple_to_js_array(t, ctx),
        ReflectRef::List(l) => reflect_list_to_js_array(l, ctx),
        ReflectRef::Array(a) => reflect_array_to_js_array(a, ctx),
        ReflectRef::Map(m) => reflect_map_to_js_map(m, ctx),
        ReflectRef::Enum(e) => reflect_enum_to_js_value(e, ctx),
        ReflectRef::Value(v) => primitive_to_js_value(v, ctx),
    }
}

fn reflect_to_js_object(reflect_struct: &dyn Struct, ctx: &mut Context) -> JsResult<JsValue> {
    let mut obj = reflect_struct
        .iter_fields()
        .enumerate()
        .map(|(idx, field)| {
            let js_value = match field.reflect_ref() {
                ReflectRef::Struct(s) => reflect_to_js_object(s, ctx)?,
                ReflectRef::TupleStruct(t) => reflect_tuple_struct_to_js_array(t, ctx)?,
                ReflectRef::Tuple(t) => reflect_tuple_to_js_array(t, ctx)?,
                ReflectRef::List(l) => reflect_list_to_js_array(l, ctx)?,
                ReflectRef::Array(a) => reflect_array_to_js_array(a, ctx)?,
                ReflectRef::Map(m) => reflect_map_to_js_map(m, ctx)?,
                ReflectRef::Enum(e) => reflect_enum_to_js_value(e, ctx)?,
                ReflectRef::Value(v) => primitive_to_js_value(v, ctx)?,
            };
            let field_name = reflect_struct
                .name_at(idx)
                .ok_or_else(|| JsError::from_opaque(js_str!("Could not read field").into()))?;
            Ok((JsString::from(field_name), js_value))
        })
        .collect::<JsResult<Vec<(JsString, JsValue)>>>()?
        .into_iter()
        .fold(ObjectInitializer::new(ctx), |mut obj, (k, v)| {
            obj.property(k, v, Attribute::all());
            obj
        });

    Ok(obj.build().into())
}

fn reflect_tuple_struct_to_js_array(
    tuple: &dyn TupleStruct,
    context: &mut Context,
) -> JsResult<JsValue> {
    let array = JsArray::new(context);
    for field in tuple.iter_fields() {
        let js_value = reflect_to_js_value(field, context)?;
        array.push(js_value, context)?;
    }
    Ok(array.into())
}

fn reflect_tuple_to_js_array(tuple: &dyn Tuple, context: &mut Context) -> JsResult<JsValue> {
    let array = JsArray::new(context);
    for field in tuple.iter_fields() {
        let js_value = reflect_to_js_value(field, context)?;
        array.push(js_value, context)?;
    }
    Ok(array.into())
}

fn reflect_list_to_js_array(list: &dyn List, context: &mut Context) -> JsResult<JsValue> {
    let array = JsArray::new(context);
    for item in list.iter() {
        let js_value = reflect_to_js_value(item, context)?;
        array.push(js_value, context)?;
    }
    Ok(array.into())
}

fn reflect_array_to_js_array(array: &dyn Array, context: &mut Context) -> JsResult<JsValue> {
    let js_array = JsArray::new(context);
    for i in 0..array.len() {
        let item = array.get(i).unwrap();
        let js_value = reflect_to_js_value(item, context)?;
        js_array.push(js_value, context)?;
    }
    Ok(js_array.into())
}

fn reflect_map_to_js_map(map: &dyn Map, context: &mut Context) -> JsResult<JsValue> {
    let js_map = JsMap::new(context);
    for (key, value) in map.iter() {
        let key_value = reflect_to_js_value(key, context)?;
        let value_value = reflect_to_js_value(value, context)?;
        js_map.set(key_value, value_value, context)?;
    }
    Ok(js_map.into())
}

fn reflect_enum_to_js_value(enum_value: &dyn Enum, context: &mut Context) -> JsResult<JsValue> {
    let variant_name = enum_value.variant_name();
    let mut obj = enum_value
        .iter_fields()
        .map(|field_value| {
            let name = field_value
                .name()
                .ok_or_else(|| JsError::from_opaque(js_str!("Could not read field name").into()))?;
            let js_value = reflect_to_js_value(field_value.value(), context)?;
            let js_str = JsString::from(name);
            Ok((js_str, js_value))
        })
        .collect::<JsResult<Vec<(JsString, JsValue)>>>()?
        .iter()
        .fold(ObjectInitializer::new(context), |mut obj, (k, v)| {
            obj.property(k.clone(), v.clone(), Attribute::all());
            obj
        });

    obj.property(
        js_str!("__variant"),
        JsValue::String(variant_name.into()),
        Attribute::all(),
    );
    Ok(obj.build().into())
}

fn primitive_to_js_value(value: &dyn Reflect, _context: &mut Context) -> JsResult<JsValue> {
    let value = value.try_as_reflect().ok_or_else(|| {
        JsError::from_opaque(js_str!("Could not convert value to reflect").into())
    })?;
    Ok(match value {
        v if v.is::<bool>() => JsValue::Boolean(*v.downcast_ref::<bool>().unwrap()),
        v if v.is::<i8>() => JsValue::Integer(*v.downcast_ref::<i8>().unwrap() as i32),
        v if v.is::<i16>() => JsValue::Integer(*v.downcast_ref::<i16>().unwrap() as i32),
        v if v.is::<i32>() => JsValue::Integer(*v.downcast_ref::<i32>().unwrap()),
        v if v.is::<i64>() => JsValue::BigInt((*v.downcast_ref::<i64>().unwrap()).into()),
        v if v.is::<isize>() => {
            JsValue::BigInt((*v.downcast_ref::<isize>().unwrap() as i64).into())
        }
        v if v.is::<u8>() => JsValue::Integer(*v.downcast_ref::<u8>().unwrap() as i32),
        v if v.is::<u16>() => JsValue::Integer(*v.downcast_ref::<u16>().unwrap() as i32),
        v if v.is::<u32>() => JsValue::BigInt((*v.downcast_ref::<u32>().unwrap() as u64).into()),
        v if v.is::<u64>() => JsValue::BigInt((*v.downcast_ref::<u64>().unwrap()).into()),
        v if v.is::<usize>() => {
            JsValue::BigInt((*v.downcast_ref::<usize>().unwrap() as u64).into())
        }
        v if v.is::<f32>() => JsValue::Rational(*v.downcast_ref::<f32>().unwrap() as f64),
        v if v.is::<f64>() => JsValue::Rational(*v.downcast_ref::<f64>().unwrap()),
        v if v.is::<String>() => {
            JsValue::String(v.downcast_ref::<String>().unwrap().clone().into())
        }
        v if v.is::<&str>() => JsValue::String((*v.downcast_ref::<&str>().unwrap()).into()),
        _ => JsValue::Null,
    })
}
