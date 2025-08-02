use std::collections::HashMap;

#[derive(Eq, PartialEq,Hash,Debug)]
pub enum OptType {
    STRING,
    BOOL,
    UNKNOWN,
}
#[derive(Eq, PartialEq,Hash,Debug)]
pub enum OptValue {
    String(String),
    BOOL(bool),
    UNKNOWN,
}
/*
     SPath="main.mosa"
 */
pub fn parseopts(argstr: Vec<String>) -> HashMap<String, OptValue> {
    let mut opts = HashMap::new();
    for opt in argstr {
        match opt.as_bytes()[1] as char {
            'S' => {
                let option = OptValue::String(opt[opt.find('=').expect("Opt does not contains val declaration")+1..].to_string());
                opts.insert(opt[2..opt.find('=').expect("Opt does not contains val declaration")].to_string(), option);
            },
            'B' => {
                let option = OptValue::BOOL(opt[opt.find('=').expect("Opt does not contains val declaration")+1..].trim().parse::<bool>().unwrap());
                opts.insert(opt[2..opt.find('=').expect("Opt does not contains val declaration")].to_string(), option );
            },
            _a => {panic!("Unknown opt type: {}", _a)}
        };
    }
    dbg!(&opts);
    opts
}