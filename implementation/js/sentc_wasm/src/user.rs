use alloc::format;
use alloc::string::String;

use sentc_crypto::user;
use wasm_bindgen::prelude::*;
use web_sys::{RequestInit, RequestMode};

use crate::make_req;

#[wasm_bindgen]
pub struct DoneLoginData
{
	private_key: String, //Base64 exported keys
	public_key: String,
	sign_key: String,
	verify_key: String,
	jwt: String,
	exported_public_key: String,
	exported_verify_key: String,
}

#[wasm_bindgen]
impl DoneLoginData
{
	pub fn get_private_key(&self) -> String
	{
		self.private_key.clone()
	}

	pub fn get_public_key(&self) -> String
	{
		self.public_key.clone()
	}

	pub fn get_sign_key(&self) -> String
	{
		self.sign_key.clone()
	}

	pub fn get_verify_key(&self) -> String
	{
		self.verify_key.clone()
	}

	pub fn get_jwt(&self) -> String
	{
		self.jwt.clone()
	}

	pub fn get_exported_public_key(&self) -> String
	{
		self.exported_public_key.clone()
	}

	pub fn get_exported_verify_key(&self) -> String
	{
		self.exported_verify_key.clone()
	}
}

/**
# Check if the identifier is available for this app
*/
#[wasm_bindgen]
pub async fn check_user_identifier_available(base_url: String, auth_token: String, user_identifier: String) -> Result<String, JsValue>
{
	let server_input = user::prepare_check_user_identifier_available(user_identifier.as_str())?;

	let url = format!("{}/api/v1/check_user_identifier", base_url);

	let mut opts = RequestInit::new();
	opts.method("POST");
	opts.mode(RequestMode::NoCors);
	opts.body(Some(&JsValue::from_str(server_input.as_str())));

	let res = make_req(url.as_str(), auth_token.as_str(), &opts).await?;

	Ok(res)
}

/**
# Get the user input from the user client

This is used when the register endpoint should only be called from the backend and not the clients.

For full register see register()
*/
#[wasm_bindgen]
pub fn prepare_register(user_identifier: &str, password: &str) -> Result<String, JsValue>
{
	Ok(user::register(user_identifier, password)?)
}

/**
# Register a new user for the app

Do the full req incl. req.
No checking about spamming and just return the user id.
*/
#[wasm_bindgen]
pub async fn register(base_url: String, auth_token: String, user_identifier: String, password: String) -> Result<String, JsValue>
{
	let register_input = user::register(user_identifier.as_str(), password.as_str())?;

	let url = format!("{}/api/v1/register", base_url);

	let mut opts = RequestInit::new();
	opts.method("POST");
	opts.mode(RequestMode::NoCors);
	opts.body(Some(&JsValue::from_str(register_input.as_str())));

	let res = make_req(url.as_str(), auth_token.as_str(), &opts).await?;

	Ok(res)
}

/**
# Login the user to this app

Does the login requests. 1. for auth, 2nd to get the keys.

If there are more data in the backend, then it is possible to call it via the jwt what is returned by the done login request.

The other backend can validate the jwt
*/
#[wasm_bindgen]
pub async fn login(base_url: String, auth_token: String, user_identifier: String, password: String) -> Result<DoneLoginData, JsValue>
{
	let user_id_input = user::prepare_login_start(user_identifier.as_str())?;

	let url = format!("{}/api/v1/pre_login", base_url);

	let mut opts = RequestInit::new();
	opts.method("POST");
	opts.mode(RequestMode::NoCors);
	opts.body(Some(&JsValue::from_str(user_id_input.as_str())));

	let res = make_req(url.as_str(), auth_token.as_str(), &opts).await?;

	//prepare the login, the auth key is already in the right json format for the server
	let (auth_key, master_key_encryption_key) = user::prepare_login(password.as_str(), res.as_str())?;

	let url = format!("{}/api/v1/login", base_url);

	//send the auth key to the server
	let mut opts = RequestInit::new();
	opts.method("POST");
	opts.mode(RequestMode::NoCors);
	opts.body(Some(&JsValue::from_str(auth_key.as_str())));

	//the done login server output
	let server_output = make_req(url.as_str(), auth_token.as_str(), &opts).await?;

	let keys = user::done_login(master_key_encryption_key.as_str(), server_output.as_str())?;

	Ok(DoneLoginData {
		private_key: keys.private_key,
		public_key: keys.public_key,
		sign_key: keys.sign_key,
		verify_key: keys.verify_key,
		jwt: keys.jwt,
		exported_public_key: keys.exported_public_key,
		exported_verify_key: keys.exported_verify_key,
	})
}
