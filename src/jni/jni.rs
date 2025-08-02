use std::string::String;
use std::collections::HashMap;
use std::sync::Mutex;
use jni::{JNIEnv, JavaVM};
use lazy_static::lazy_static;

pub fn assign_global_jvm(jvm: JNIEnv) {
    GLOBAL.lock().unwrap().insert("PUBLIC".parse().unwrap(), jvm.get_java_vm().unwrap());
}
pub fn jvm() {
    GLOBAL.lock().unwrap().get("PUBLIC").unwrap();
}

lazy_static! {
    static ref GLOBAL: Mutex<HashMap<String,JavaVM>> = Mutex::new(HashMap::new());
}
#[derive(Clone)]
pub struct JNIClass {
    name: String,
    modifiers: Modifier,

    path: String,
    fields: Vec<JNIField>,
    methods: Vec<JNIMethod>,
    constructors: Vec<JNIConstructor>,
}
#[derive(Clone)]
pub struct Modifier {
    modifiers: i32,
}
#[derive(Clone)]
pub struct JNIMethod {
    // commons
    owner: JNIClass,
    name: String,
    modifiers: Modifier,
    // commons
    return_types: Vec<JNIClass>,
    parameter_types: Vec<JNIClass>,
    exception_types: Vec<JNIClass>
}
#[derive(Clone)]
pub struct JNIField {
    // commons
    owner: JNIClass,
    name: String,
    modifiers: Modifier
    // commons
}
#[derive(Clone)]
pub struct JNIConstructor {
    // commons
    owner: JNIClass,
    name: String,
    modifiers: Modifier,
    // commons
    parameter_types: Vec<JNIClass>,
    exception_types: Vec<JNIClass>
}
pub fn staticmodifier() -> Modifier {
    Modifier {
        modifiers: 8
    }
}

impl Modifier {
    /*
    public static final int PUBLIC = 1;
    public static final int PRIVATE = 2;
    public static final int PROTECTED = 4;
    public static final int STATIC = 8;
    public static final int FINAL = 16;
    public static final int SYNCHRONIZED = 32;
    public static final int VOLATILE = 64;
    public static final int TRANSIENT = 128;
    public static final int NATIVE = 256;
    public static final int INTERFACE = 512;
    public static final int ABSTRACT = 1024;
    public static final int STRICT = 2048;
    public static final int SYNTHETIC = 4096;
    public static final int ANNOTATION = 8192;
    public static final int ENUM = 16384;
    public static final int MANDATED = 32768;
     */
    fn mod_eq(&self,id: i32) -> bool {
        (self.modifiers & id) != 0
    }
    pub fn is_public(&self) -> bool {
        self.mod_eq(1)
    }
    pub fn is_private(&self) -> bool {
        self.mod_eq(2)
    }
    pub fn is_protected(&self) -> bool {
        self.mod_eq(4)
    }
    pub fn is_static(&self) -> bool {
        self.mod_eq(8)
    }
    pub fn is_final(&self) -> bool {
        self.mod_eq(16)
    }
    pub fn is_synchronized(&self) -> bool {
        self.mod_eq(32)
    }
    pub fn is_volatile(&self) -> bool {
        self.mod_eq(64)
    }
    pub fn is_transient(&self) -> bool {
        self.mod_eq(128)
    }
    pub fn is_native(&self) -> bool {
        self.mod_eq(256)
    }
    pub fn is_interface(&self) -> bool {
        self.mod_eq(512)
    }
    pub fn is_abstract(&self) -> bool {
        self.mod_eq(1024)
    }
    pub fn is_strict(&self) -> bool {
        self.mod_eq(2048)
    }
    pub fn is_synthetic(&self) -> bool {
        self.mod_eq(4096)
    }
    pub fn is_annotation(&self) -> bool {
        self.mod_eq(8192)
    }
    pub fn is_enum(&self) -> bool {
        self.mod_eq(16384)
    }
    pub fn is_mandated(&self) -> bool {
        self.mod_eq(32768)
    }
}