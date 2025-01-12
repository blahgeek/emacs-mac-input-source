use emacs::{Env, IntoLisp, Result, Value};

pub fn property_name_to_lisp<'e, 's>(env: &'e Env, s: &'s str) -> Result<Value<'e>> {
    let mut s = s.trim_end_matches("_").replace("_", "-");
    if s.starts_with("is-") {
        s = (&s[3..]).to_string() + "-p";
    }
    env.intern(&format!(":{}", &s))
}

pub fn vec_into_lisp<'e, T>(env: &'e Env, vec: Option<Vec<T>>) -> Result<Value<'e>>
where T: IntoLisp<'e> {
    if let Some(vec) = vec {
        let mut result = ().into_lisp(env)?;
        for v in vec {
            let v_lisp = v.into_lisp(env)?;
            result = env.cons(v_lisp, result)?;
        }
        Ok(result)
    } else {
        ().into_lisp(env)
    }
}
