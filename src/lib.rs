use worker::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct InviteRequest {
    invite_code: String,
}

#[derive(Deserialize, Serialize)]
struct RsvpForm {
    invite_code: String,
    guest_name: String,
    attending: bool,
    meal: String,
    special_requests: Option<String>,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new()
        // POST: Validate invite code
        .post_async("/validate-invite", |mut req, ctx| async move {
            let form: Result<InviteRequest> = req.json().await;
            let form = match form {
                Ok(f) => f,
                Err(_) => {
                    return Response::from_json(&serde_json::json!({
                        "status": "error",
                        "message": "Invalid request format."
                    })).map_err(Into::into);
                }
            };

            let invite_code = &form.invite_code;
            let kv = ctx.env.kv("INVITE_CODES")?;
            let value: Option<String> = match kv.get(invite_code).text().await {
                Ok(val) => val,
                Err(_) => {
                    return Response::from_json(&serde_json::json!({
                        "status": "error",
                        "message": "Error retrieving invite code."
                    })).map_err(Into::into);
                }
            };

            let response = match value {
                Some(val) if val == "valid" => serde_json::json!({
                    "status": "success",
                    "message": "Invite code is valid. Please submit your RSVP."
                }),
                Some(_) => serde_json::json!({
                    "status": "error",
                    "message": "This invite code has already been used or is invalid."
                }),
                None => serde_json::json!({
                    "status": "error",
                    "message": "Invalid invite code."
                })
            };

            Response::from_json(&response).map_err(Into::into)
        })
        
        .post_async("/submit-rsvp", |mut req, ctx| async move {
    // Parse the incoming request body to get the RSVP form.
    let rsvp_form: Result<RsvpForm> = req.json().await;
    let rsvp_form = match rsvp_form {
        Ok(f) => f,
        Err(_) => {
            return Response::from_json(&serde_json::json!({
                "status": "error",
                "message": "Invalid RSVP format."
            })).map_err(Into::into);
        }
    };

    // Use invite_code from the form as the unique identifier for the RSVP.
    let invite_code = &rsvp_form.invite_code;  // Now using invite_code instead of guest_name
    let kv = ctx.env.kv("RSVP_DETAILS")?;

    // Check if an RSVP entry already exists for this invite_code.
    let existing_rsvp: Option<String> = match kv.get(invite_code).text().await {
        Ok(val) => val,
        Err(_) => {
            return Response::from_json(&serde_json::json!({
                "status": "error",
                "message": "Error checking RSVP status."
            })).map_err(Into::into);
        }
    };

    // Check if the RSVP already exists or not.
    let json_response = match existing_rsvp {
        Some(_) => {
            // If the RSVP already exists, update it.
            match kv.put(invite_code, serde_json::to_string(&rsvp_form)?)?.execute().await {
                Ok(_) => serde_json::json!({
                    "status": "success",
                    "message": "RSVP updated successfully!"
                }),
                Err(_) => serde_json::json!({
                    "status": "error",
                    "message": "Error saving RSVP."
                }),
            }
        },
        None => {
            // If the RSVP doesn't exist, store it.
            match kv.put(invite_code, serde_json::to_string(&rsvp_form)?)?.execute().await {
                Ok(_) => serde_json::json!({
                    "status": "success",
                    "message": "RSVP submitted successfully!"
                }),
                Err(_) => serde_json::json!({
                    "status": "error",
                    "message": "Error saving RSVP."
                }),
            }
        }
    };

    // Return the response.
    Response::from_json(&json_response).map_err(Into::into)
})


        // GET: View profile for invite code
        .get_async("/profile/:invite_code", |_req, ctx| async move {
            let invite_code = ctx.param("invite_code").ok_or_else(|| worker::Error::RustError("Missing invite_code".into()))?.to_string();
            let kv = ctx.env.kv("RSVP_DETAILS")?;

            let guest_data: Option<String> = match kv.get(&invite_code).text().await {
                Ok(val) => val,
                Err(_) => {
                    return Response::from_json(&serde_json::json!({
                        "status": "error",
                        "message": "Error retrieving guest data."
                    })).map_err(Into::into);
                }
            };

            match guest_data {
                Some(data) => {
                    let profile: RsvpForm = match serde_json::from_str(&data) {
                        Ok(p) => p,
                        Err(_) => {
                            return Response::from_json(&serde_json::json!({
                                "status": "error",
                                "message": "Error deserializing profile data."
                            })).map_err(Into::into);
                        }
                    };
                    Response::from_json(&profile).map_err(Into::into)
                },
                None => {
                    Response::from_json(&serde_json::json!({
                        "status": "error",
                        "message": "Profile not found."
                    })).map_err(Into::into)
                }
            }
        });

    router.run(req, env).await
}
