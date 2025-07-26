use crate::jni::jni::Modifier;
use jni::objects::{JObject, JValue, JValueOwned};
use jni::sys::JavaVM;
use jni::JNIEnv;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
#[derive(Clone)]
pub struct Callable {
    classpath: String,
    method: String,
    descriptor: String,
    modifiers: Modifier,
}
impl Callable {
    pub unsafe fn new(
        id: String,
        path: String,
        method: String,
        descriptor: String,
        modifier: Modifier,
    ) -> Callable {
        let mut callable = Callable {
            classpath: path,
            method,
            descriptor,
            modifiers: modifier,
        };
        cache_add(id, callable.clone());
        callable
    }
    pub fn call<'local>(
        self,
        mut jni: JNIEnv<'local>,
        args: &[JValue],
        obj: Option<JObject>,
    ) -> jni::errors::Result<JValueOwned<'local>> {
        match self.modifiers.is_static() {
            true => jni.call_static_method(self.classpath, self.method, self.descriptor, args),
            false => jni.call_method(obj.unwrap(), self.method, self.descriptor, args),
        }
    }
}
pub unsafe fn cache_add(id: String, cal: Callable) {
    CALLABLE_CACHE.lock().unwrap().insert(id, cal);
}
pub unsafe fn cache_get(cal: &String) {
    CALLABLE_CACHE.lock().unwrap().get(cal);
}
lazy_static! {
    static ref CALLABLE_CACHE: Mutex<HashMap<String, Callable>> = Mutex::new(HashMap::new());
}
