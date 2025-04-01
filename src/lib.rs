use worker::*; use uuid::Uuid; use wasm_bindgen::JsValue; use serde_json::Value;

fn cors_headers() -> Headers { let mut headers = Headers::new(); headers.set("Access-Control-Allow-Origin", "http://localhost:5173").unwrap(); headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap(); headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization").unwrap(); headers.set("Access-Control-Allow-Credentials", "true").unwrap(); headers }

#[event(fetch)] pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> { if req.method() == Method::Options { let mut res = Response::empty()?; for (key, value) in cors_headers().entries() { res.headers_mut().set(&key, &value)?; } return Ok(res); }

let router = Router::new()
//    .post_async("/login", |mut req, ctx| async move {
//        let kv = ctx.kv("INVITE_CODES")?;
//        let d1 = ctx.d1("DB")?;
//        let body: Value = req.json().await?;
//        //let invite_code = body["invite_code"].as_str().unwrap_or("").to_string();
//        let invite_code = body.get("invite_code").and_then(|v| v.as_str()).unwrap_or("").to_string();
//        // Validate invite code
//        let kv_list = kv.list().execute().await?;
//        if !kv_list.keys.iter().any(|k| k.name == invite_code) {
//            return Response::error("Invalid invite code", 403);
//        }
//
//        // Generate session token
//        let token = Uuid::new_v4().to_string();
//        let stmt = d1.prepare("INSERT INTO sessions (invite_code, token) VALUES (?, ?)")
//            .bind(&[JsValue::from(invite_code), JsValue::from(token.clone())])?;
//        stmt.run().await?;
//
//        let mut res = Response::ok("Logged in")?;
//        res.headers_mut().set("Set-Cookie", &format!("session={}; HttpOnly; Path=/", token))?;
//        for (key, value) in cors_headers().entries() {
//            res.headers_mut().set(&key, &value)?;
//        }
//        Ok(res)
//    })
    .post_async("/login", |mut req, ctx| async move {
            let kv = ctx.kv("INVITE_CODES")?;
            let d1 = ctx.d1("DB")?;
            let body: Value = req.json().await?;
            let invite_code = body["invite_code"].as_str().unwrap_or("").to_string();

            let kv_list = kv.list().execute().await?;
            if !kv_list.keys.iter().any(|k| k.name == invite_code) {
                return Response::error("Invalid invite code", 403);
            }

            let token = Uuid::new_v4().to_string();
            let stmt = d1.prepare("INSERT INTO sessions (invite_code, token) VALUES (?, ?)")
                .bind(&[JsValue::from(invite_code), JsValue::from(token.clone())])?;
            stmt.run().await?;

            //let mut res = Response::redirect(Url::parse("https://vows-api.clydecreta.workers.dev/profile")?)?;
            let base_url = "https://vows-api.clydecreta.workers.dev";
            let redirect_url = format!("{}/profile", base_url);
            let url = Url::parse(&redirect_url)?;
            let mut res = Response::redirect(url);
            res.headers_mut().set("Set-Cookie", &format!("session={}; HttpOnly; Path=/", token))?;
            Ok(res)
        })
//    .get_async("/profile", |req, ctx| async move {
//        let d1 = ctx.d1("DB")?;
//        let token = req.headers().get("Cookie")?.and_then(|c| c.split('=').nth(1).map(String::from));
//
//        match token {
//            Some(t) => {
//                let stmt = d1.prepare("SELECT invite_code FROM sessions WHERE token = ?")
//                    .bind(&[JsValue::from(t)])?;
//                let result = stmt.first::<(String,)>(None).await?;
//                if let Some((invite_code,)) = result {
//                    let mut res = Response::ok(format!("Welcome, user with invite code: {}", invite_code))?;
//                    for (key, value) in cors_headers().entries() {
//                        res.headers_mut().set(&key, &value)?;
//                    }
//                    return Ok(res);
//                }
//            }
//            None => {}
//        }
//        let mut res = Response::error("Unauthorized", 401)?;
//        for (key, value) in cors_headers().entries() {
//            res.headers_mut().set(&key, &value)?;
//        }
//        Ok(res)
//    })
    .get_async("/profile", |req, ctx| async move {
    let d1 = ctx.d1("DB")?;
    let token = req.headers().get("Cookie")?
        .and_then(|c| c.split('=').nth(1).map(String::from));

    match token {
        Some(t) => {
            let stmt = d1.prepare("SELECT invite_code FROM sessions WHERE token = ?")
                .bind(&[JsValue::from(t)])?;
            let result = stmt.first::<(String,)>(None).await?;
            if let Some((invite_code,)) = result {
                let mut res = Response::ok(format!("Welcome, user with invite code: {}", invite_code))?;
                //res.headers_mut().extend(cors_headers());
		for (key, value) in cors_headers().entries() {
    		    res.headers_mut().set(&key, &value)?;
		}
                return Ok(res);
            }
        }
        None => {}
    }

    let mut res = Response::error("Unauthorized", 401)?;
    //res.headers_mut().extend(cors_headers());
    for (key, value) in cors_headers().entries() {
        res.headers_mut().set(&key, &value)?;
    }
    Ok(res)
    })
    .post_async("/logout", |req, ctx| async move {
        let d1 = ctx.d1("DB")?;
        let token = req.headers().get("Cookie")?.and_then(|c| c.split('=').nth(1).map(String::from));

        if let Some(t) = token {
            let stmt = d1.prepare("DELETE FROM sessions WHERE token = ?")
                .bind(&[JsValue::from(t)])?;
            stmt.run().await?;
            let mut res = Response::ok("Logged out")?;
            res.headers_mut().set("Set-Cookie", "session=; HttpOnly; Path=/; Max-Age=0")?;
            for (key, value) in cors_headers().entries() {
                res.headers_mut().set(&key, &value)?;
            }
            return Ok(res);
        }
        let mut res = Response::error("Not logged in", 400)?;
        for (key, value) in cors_headers().entries() {
            res.headers_mut().set(&key, &value)?;
        }
        Ok(res)
    })
    .run(req, env).await?;

let mut response = router;
for (key, value) in cors_headers().entries() {
    response.headers_mut().set(&key, &value)?;
}
Ok(response)

}
