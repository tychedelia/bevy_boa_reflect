use bevy::reflect::Reflect;
use boa_engine::{Context, JsResult, JsValue};

mod from;
mod into;

/// Trait for converting a type into a `JsValue`.
pub trait IntoJsValue {
    /// Convert the type into a `JsValue`, panicking if the conversion fails.
    fn into_js_value(self, ctx: &mut Context) -> JsValue;

    /// Convert the type into a `JsValue`, returning an error if the conversion fails.
    fn try_into_js_value(self, ctx: &mut Context) -> JsResult<JsValue>;
}

impl<T> IntoJsValue for T
where
    T: Reflect,
{
    fn into_js_value(self, ctx: &mut Context) -> JsValue {
        into::reflect_to_js_value(&self, ctx).unwrap()
    }

    fn try_into_js_value(self, ctx: &mut Context) -> JsResult<JsValue> {
        into::reflect_to_js_value(&self, ctx)
    }
}

/// Trait for converting a [`JsValue`] into a type.
pub trait FromJsValue {
    /// Convert a `JsValue` into the type, panicking if the conversion fails.
    fn from_js_value(value: JsValue, ctx: &mut Context) -> Self;

    /// Convert a `JsValue` into the type, returning an error if the conversion fails.
    fn try_from_js_value(value: JsValue, ctx: &mut Context) -> JsResult<Self>;
}

impl<T> FromJsValue for T
where
    T: Reflect,
{
    fn from_js_value(value: JsValue, ctx: &mut Context) -> Self {
        from::js_value_to_reflect(value, ctx).unwrap()
    }

    fn try_from_js_value(value: JsValue, ctx: &mut Context) -> JsResult<Self> {
        from::js_value_to_reflect(value, ctx)
    }
}
