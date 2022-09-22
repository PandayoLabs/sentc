#[cfg(not(feature = "rust"))]
mod non_rust;
#[cfg(feature = "rust")]
mod rust;

use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;

use sentc_crypto::user;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto_common::group::GroupAcceptJoinReqServerOutput;
use sentc_crypto_common::user::{DoneLoginLightServerOutput, UserDeviceList, UserInitServerOutput};

#[cfg(not(feature = "rust"))]
pub(crate) use self::non_rust::{
	BoolRes,
	DeviceListRes,
	InitRes,
	LoginRes,
	Res,
	SessionRes,
	UserKeyFetchRes,
	UserPublicDataRes,
	UserPublicKeyRes,
	UserVerifyKeyRes,
	VoidRes,
};
#[cfg(feature = "rust")]
pub(crate) use self::rust::{
	BoolRes,
	DeviceListRes,
	InitRes,
	LoginRes,
	Res,
	SessionRes,
	UserKeyFetchRes,
	UserPublicDataRes,
	UserPublicKeyRes,
	UserVerifyKeyRes,
	VoidRes,
};
use crate::group;
use crate::group::KeyRotationRes;
use crate::util::{make_non_auth_req, make_req, HttpMethod};

//Register
pub async fn check_user_identifier_available(base_url: String, auth_token: &str, user_identifier: &str) -> BoolRes
{
	let server_input = user::prepare_check_user_identifier_available(user_identifier)?;

	let url = base_url + "/api/v1/exists";

	let res = make_non_auth_req(HttpMethod::POST, url.as_str(), auth_token, Some(server_input)).await?;
	let out = user::done_check_user_identifier_available(res.as_str())?;

	Ok(out)
}

pub async fn register(base_url: String, auth_token: &str, user_identifier: &str, password: &str) -> Res
{
	let register_input = user::register(user_identifier, password)?;

	let url = base_url + "/api/v1/register";

	let res = make_non_auth_req(HttpMethod::POST, url.as_str(), auth_token, Some(register_input)).await?;

	let out = user::done_register(res.as_str())?;

	Ok(out)
}

pub async fn register_device_start(base_url: String, auth_token: &str, device_identifier: &str, password: &str) -> Res
{
	let url = base_url + "/api/v1/user/prepare_register_device";

	let input = user::prepare_register_device_start(device_identifier, password)?;

	let res = make_non_auth_req(HttpMethod::POST, url.as_str(), auth_token, Some(input)).await?;

	//check the server output
	user::done_register_device_start(res.as_str())?;

	Ok(res)
}

pub async fn register_device(
	base_url: String,
	auth_token: &str,
	jwt: &str,
	server_output: &str,
	key_count: i32,
	#[cfg(not(feature = "rust"))] user_keys: &str,
	#[cfg(feature = "rust")] user_keys: &[&sentc_crypto::util::SymKeyFormat],
) -> SessionRes
{
	let url = base_url + "/api/v1/user/done_register_device";

	let key_session = if key_count > 50 { true } else { false };

	let input = user::prepare_register_device(server_output, user_keys, key_session)?;

	let res = make_req(HttpMethod::PUT, url.as_str(), auth_token, Some(input), Some(jwt)).await?;

	let out: GroupAcceptJoinReqServerOutput = handle_server_response(res.as_str())?;

	Ok(out.session_id)
}

//__________________________________________________________________________________________________
//Login

pub async fn prepare_login_start(base_url: String, auth_token: &str, user_identifier: &str) -> Res
{
	let user_id_input = user::prepare_login_start(user_identifier)?;

	let url = base_url + "/api/v1/prepare_login";

	let res = make_non_auth_req(HttpMethod::POST, url.as_str(), auth_token, Some(user_id_input)).await?;

	Ok(res)
}

pub async fn done_login(base_url: String, auth_token: &str, user_identifier: &str, password: &str, prepare_login_server_output: &str) -> LoginRes
{
	let (auth_key, master_key_encryption_key) = user::prepare_login(user_identifier, password, prepare_login_server_output)?;

	let url = base_url + "/api/v1/done_login";

	let server_out = make_non_auth_req(HttpMethod::POST, url.as_str(), auth_token, Some(auth_key)).await?;

	let keys = user::done_login(&master_key_encryption_key, server_out.as_str())?;

	Ok(keys)
}

pub async fn login(base_url: String, auth_token: &str, user_identifier: &str, password: &str) -> LoginRes
{
	let user_id_input = user::prepare_login_start(user_identifier)?;

	let url = base_url.clone() + "/api/v1/prepare_login";

	let res = make_non_auth_req(HttpMethod::POST, url.as_str(), auth_token, Some(user_id_input)).await?;

	//prepare the login, the auth key is already in the right json format for the server
	let (auth_key, master_key_encryption_key) = user::prepare_login(user_identifier, password, res.as_str())?;

	let url = base_url + "/api/v1/done_login";

	let server_out = make_non_auth_req(HttpMethod::POST, url.as_str(), auth_token, Some(auth_key)).await?;

	let keys = user::done_login(&master_key_encryption_key, server_out.as_str())?;

	Ok(keys)
}

pub async fn fetch_user_key(
	base_url: String,
	auth_token: &str,
	jwt: &str,
	key_id: &str,
	#[cfg(not(feature = "rust"))] private_key: &str,
	#[cfg(feature = "rust")] private_key: &sentc_crypto::util::PrivateKeyFormat,
) -> UserKeyFetchRes
{
	let url = base_url + "/api/v1/user/user_keys/key/" + key_id;

	let server_out = make_req(HttpMethod::GET, url.as_str(), auth_token, None, Some(jwt)).await?;

	let keys = user::done_key_fetch(private_key, server_out.as_str())?;

	Ok(keys)
}

pub async fn refresh_jwt(base_url: String, auth_token: &str, jwt: &str, refresh_token: &str) -> Res
{
	let input = user::prepare_refresh_jwt(refresh_token)?;

	let url = base_url + "/api/v1/refresh";

	let res = make_req(HttpMethod::PUT, url.as_str(), auth_token, Some(input), Some(jwt)).await?;

	let server_output: DoneLoginLightServerOutput = handle_server_response(res.as_str())?;

	Ok(server_output.jwt)
}

pub async fn init_user(base_url: String, auth_token: &str, jwt: &str, refresh_token: &str) -> InitRes
{
	let input = user::prepare_refresh_jwt(refresh_token)?;

	let url = base_url + "/api/v1/init";

	let res = make_req(HttpMethod::POST, url.as_str(), auth_token, Some(input), Some(jwt)).await?;

	let server_output: UserInitServerOutput = handle_server_response(res.as_str())?;

	Ok(server_output)
}

pub async fn get_user_devices(base_url: String, auth_token: &str, jwt: &str, last_fetched_time: &str, last_fetched_id: &str) -> DeviceListRes
{
	let url = base_url + "/api/v1/user/device/" + last_fetched_time + "/" + last_fetched_id;

	let res = make_req(HttpMethod::GET, url.as_str(), auth_token, None, Some(jwt)).await?;

	let out: Vec<UserDeviceList> = handle_server_response(res.as_str())?;

	Ok(out)
}

//__________________________________________________________________________________________________

pub async fn change_password(base_url: String, auth_token: &str, user_identifier: &str, old_password: &str, new_password: &str) -> VoidRes
{
	//first make the prep login req to get the output
	let prep_login_out = prepare_login_start(base_url.clone(), auth_token, user_identifier).await?;

	let (auth_key, master_key_encryption_key) = user::prepare_login(user_identifier, old_password, prep_login_out.as_str())?;

	//make done login req again to get a fresh jwt
	let url = base_url.clone() + "/api/v1/done_login";

	let done_login_out = make_non_auth_req(HttpMethod::POST, url.as_str(), auth_token, Some(auth_key)).await?;

	let keys = user::done_login(&master_key_encryption_key, done_login_out.as_str())?;

	let change_pw_input = user::change_password(
		old_password,
		new_password,
		prep_login_out.as_str(),
		done_login_out.as_str(),
	)?;

	let url = base_url + "/api/v1/user/update_pw";

	let res = make_req(
		HttpMethod::PUT,
		url.as_str(),
		auth_token,
		Some(change_pw_input),
		Some(keys.jwt.as_str()),
	)
	.await?;

	Ok(handle_general_server_response(res.as_str())?)
}

pub async fn reset_password(
	base_url: String,
	auth_token: &str,
	jwt: &str,
	new_password: &str,
	#[cfg(not(feature = "rust"))] decrypted_private_key: &str,
	#[cfg(not(feature = "rust"))] decrypted_sign_key: &str,
	#[cfg(feature = "rust")] decrypted_private_key: &sentc_crypto::util::PrivateKeyFormat,
	#[cfg(feature = "rust")] decrypted_sign_key: &sentc_crypto::util::SignKeyFormat,
) -> VoidRes
{
	let url = base_url + "/api/v1/user/reset_pw";

	let input = user::reset_password(new_password, decrypted_private_key, decrypted_sign_key)?;

	let res = make_req(HttpMethod::PUT, url.as_str(), auth_token, Some(input), Some(jwt)).await?;

	Ok(handle_general_server_response(res.as_str())?)
}

//__________________________________________________________________________________________________

pub async fn delete(base_url: String, auth_token: &str, user_identifier: &str, password: &str) -> VoidRes
{
	let prep_login_out = prepare_login_start(base_url.clone(), auth_token, user_identifier).await?;

	let (auth_key, master_key_encryption_key) = user::prepare_login(user_identifier, password, prep_login_out.as_str())?;

	//make done login req again to get a fresh jwt
	let url = base_url.clone() + "/api/v1/done_login";

	let done_login_out = make_non_auth_req(HttpMethod::POST, url.as_str(), auth_token, Some(auth_key)).await?;

	let keys = user::done_login(&master_key_encryption_key, done_login_out.as_str())?;

	let url = base_url + "/api/v1/user";

	let res = make_req(
		HttpMethod::DELETE,
		url.as_str(),
		auth_token,
		None,
		Some(keys.jwt.as_str()),
	)
	.await?;

	Ok(handle_general_server_response(res.as_str())?)
}

/**
# Remove a device from the user group.

This can only be done when the actual device got a fresh jwt,
to make sure that no hacker can remove devices.
*/
pub async fn delete_device(base_url: String, auth_token: &str, device_identifier: &str, password: &str, device_id: &str) -> VoidRes
{
	let prep_login_out = prepare_login_start(base_url.clone(), auth_token, device_identifier).await?;

	let (auth_key, master_key_encryption_key) = user::prepare_login(device_identifier, password, prep_login_out.as_str())?;

	//make done login req again to get a fresh jwt
	let url = base_url.clone() + "/api/v1/done_login";

	let done_login_out = make_non_auth_req(HttpMethod::POST, url.as_str(), auth_token, Some(auth_key)).await?;

	let keys = user::done_login(&master_key_encryption_key, done_login_out.as_str())?;

	let url = base_url + "/api/v1/user/device/" + device_id;

	let res = make_req(
		HttpMethod::DELETE,
		url.as_str(),
		auth_token,
		None,
		Some(keys.jwt.as_str()),
	)
	.await?;

	Ok(handle_general_server_response(res.as_str())?)
}

//__________________________________________________________________________________________________

pub async fn update(base_url: String, auth_token: &str, jwt: &str, user_identifier: String) -> VoidRes
{
	let url = base_url + "/api/v1/user";

	let input = user::prepare_user_identifier_update(user_identifier)?;

	let res = make_req(HttpMethod::PUT, url.as_str(), auth_token, Some(input), Some(jwt)).await?;

	Ok(handle_general_server_response(res.as_str())?)
}

//__________________________________________________________________________________________________

pub async fn fetch_user_public_data(base_url: String, auth_token: &str, user_id: &str) -> UserPublicDataRes
{
	let url = base_url + "/api/v1/user/" + user_id;

	let res = make_non_auth_req(HttpMethod::GET, url.as_str(), auth_token, None).await?;

	#[cfg(feature = "rust")]
	let public_data = sentc_crypto::util::public::import_public_data_from_string_into_format(res.as_str())?;

	#[cfg(not(feature = "rust"))]
	let public_data = sentc_crypto::util::public::import_public_data_from_string_into_export_string(res.as_str())?;

	Ok(public_data)
}

pub async fn fetch_user_public_key(base_url: String, auth_token: &str, user_id: &str) -> UserPublicKeyRes
{
	let url = base_url + "/api/v1/user/" + user_id + "/public_key";

	let res = make_non_auth_req(HttpMethod::GET, url.as_str(), auth_token, None).await?;

	#[cfg(feature = "rust")]
	let public_data = sentc_crypto::util::public::import_public_key_from_string_into_format(res.as_str())?;

	#[cfg(not(feature = "rust"))]
	let public_data = sentc_crypto::util::public::import_public_key_from_string_into_export_string(res.as_str())?;

	Ok(public_data)
}

pub async fn fetch_user_verify_key(base_url: String, auth_token: &str, user_id: &str) -> UserVerifyKeyRes
{
	let url = base_url + "/api/v1/user/" + user_id + "/verify_key";

	let res = make_non_auth_req(HttpMethod::GET, url.as_str(), auth_token, None).await?;

	#[cfg(feature = "rust")]
	let public_data = sentc_crypto::util::public::import_verify_key_from_string_into_format(res.as_str())?;

	#[cfg(not(feature = "rust"))]
	let public_data = sentc_crypto::util::public::import_verify_key_from_string_into_export_string(res.as_str())?;

	Ok(public_data)
}

//__________________________________________________________________________________________________

pub fn key_rotation<'a>(
	base_url: String,
	auth_token: &'a str,
	jwt: &'a str,
	#[cfg(not(feature = "rust"))] device_public_key: &'a str,
	#[cfg(feature = "rust")] device_public_key: &'a sentc_crypto::util::PublicKeyFormat,
	#[cfg(not(feature = "rust"))] pre_user_key: &'a str,
	#[cfg(feature = "rust")] pre_user_key: &'a sentc_crypto::util::SymKeyFormat,
) -> impl Future<Output = Res> + 'a
{
	group::key_rotation(base_url, auth_token, jwt, "", device_public_key, pre_user_key, true)
}

pub fn prepare_done_key_rotation<'a>(base_url: String, auth_token: &'a str, jwt: &'a str) -> impl Future<Output = KeyRotationRes> + 'a
{
	group::prepare_done_key_rotation(base_url, auth_token, jwt, "", true)
}

pub fn done_key_rotation<'a>(
	base_url: String,
	auth_token: &'a str,
	jwt: &'a str,
	#[cfg(not(feature = "rust"))] server_output: &'a str,
	#[cfg(feature = "rust")] server_output: &'a sentc_crypto_common::group::KeyRotationInput,
	#[cfg(not(feature = "rust"))] pre_user_key: &'a str,
	#[cfg(feature = "rust")] pre_user_key: &'a sentc_crypto::util::SymKeyFormat,
	#[cfg(not(feature = "rust"))] device_public_key: &'a str,
	#[cfg(feature = "rust")] device_public_key: &'a sentc_crypto::util::PublicKeyFormat,
	#[cfg(not(feature = "rust"))] device_private_key: &'a str,
	#[cfg(feature = "rust")] device_private_key: &'a sentc_crypto::util::PrivateKeyFormat,
) -> impl Future<Output = VoidRes> + 'a
{
	group::done_key_rotation(
		base_url,
		auth_token,
		jwt,
		"",
		server_output,
		pre_user_key,
		device_public_key,
		device_private_key,
		true,
	)
}
