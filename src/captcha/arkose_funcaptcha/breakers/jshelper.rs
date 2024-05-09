use std::str::FromStr;
use serde_json::Value;
use v8::OwnedIsolate;
use crate::commons::error::DortCapError::CodeErr;
use crate::commons::error::DortCapResult;

#[cfg(not(target_arch = "wasm32"))]
pub fn breakers(dapi_code: &str, input: Value) -> DortCapResult<Value> {
    let isolate: &mut OwnedIsolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);
    let code = r#"
        let nigger = [];
        let window = {
         document: {
           activeElement: "XD",
           requestAnimationFrame: "XD",
           cancelAnimationFrame: "XD",
           hidden: false,
           visibilityState: "prerender"
         },
         parent: {
           ae: {
             answer: REPLACE_ME,
             tanswer: [],
             dapibReceive: function(tanswer) {
               nigger.push(tanswer['tanswer']);
             }
           }
         }
        };
    "#;
    let append = format!("{}{}; JSON.stringify(nigger)", code.replace("REPLACE_ME", &*input.to_string()), dapi_code);
    let code = v8::String::new(scope, &*append).ok_or(CodeErr(0x02, "TG_API_BREAKERS"))?;
    let script = v8::Script::compile(scope, code, None).ok_or(CodeErr(0x03, "TG_API_BREAKERS"))?;
    let result = script.run(scope).ok_or(CodeErr(0x04, "TG_API_BREAKERS"))?;
    let result = result.to_string(scope).ok_or(CodeErr(0x05, "TG_API_BREAKERS"))?;
    let xd = result.to_rust_string_lossy(scope);
    return Ok(Value::from_str(&xd)?);
}