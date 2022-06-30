mod alg;
mod core;
mod error;
mod user;

use base64ct::{Base64, Encoding};

pub(crate) use self::alg::asym::{AsymKeyOutput, Pk, Sk};
#[cfg(feature = "rust")]
pub use self::alg::pw_hash::DeriveMasterKeyForAuth;
#[cfg(not(feature = "rust"))]
pub(crate) use self::alg::pw_hash::DeriveMasterKeyForAuth;
pub(crate) use self::alg::pw_hash::{
	ClientRandomValue,
	DeriveAuthKeyForAuth,
	DeriveKeyOutput,
	DeriveKeysForAuthOutput,
	HashedAuthenticationKey,
	MasterKeyInfo,
};
pub(crate) use self::alg::sign::{SignK, SignOutput, VerifyK};
pub(crate) use self::alg::sym::{SymKey, SymKeyOutput};
pub use self::error::{err_to_msg, Error};
pub use self::user::{change_password, done_login, prepare_login, register, DoneLoginOutput};
use crate::core::user::{done_login as done_login_internally, prepare_login as prepare_login_internally, register as register_internally};

pub fn aes() -> String
{
	//aes
	aes_intern()
}

fn aes_intern() -> String
{
	let test = "plaintext message";
	let test2 = "plaintext message2";

	let res = alg::sym::aes_gcm::generate_and_encrypt(test.as_ref());

	let (output, encrypted) = match res {
		Err(e) => return format!("Error for encrypt test 1: {:?}", e),
		Ok(v) => v,
	};

	let res = alg::sym::aes_gcm::encrypt(&output.key, test2.as_ref());

	let encrypted2 = match res {
		Err(e) => return format!("Error for encrypt test 2: {:?}", e),
		Ok(v) => v,
	};

	//decrypt
	let res = alg::sym::aes_gcm::decrypt(&output.key, &encrypted);

	let decrypted = match res {
		Err(e) => return format!("Error for decrypt test 1: {:?}", e),
		Ok(v) => v,
	};

	let res = alg::sym::aes_gcm::decrypt(&output.key, &encrypted2);

	let decrypted2 = match res {
		Err(e) => return format!("Error for decrypt test 2: {:?}", e),
		Ok(v) => v,
	};

	assert_eq!(&decrypted, b"plaintext message");
	assert_eq!(&decrypted2, b"plaintext message2");

	let one = std::str::from_utf8(&decrypted).unwrap().to_owned();
	let two = std::str::from_utf8(&decrypted2).unwrap();

	one + " " + two
}

pub fn ecdh() -> String
{
	// Alice
	//let (alice_secret, alice_pk) = alg::asym::ecies::generate_static_keypair();

	// Bob
	let bob_out = alg::asym::ecies::generate_static_keypair();

	let bob_secret = bob_out.sk;
	let bob_pk = bob_out.pk;

	//Alice create a msg for Bob's public key
	let alice_msg = "Hello Bob";
	let alice_encrypted = alg::asym::ecies::encrypt(&bob_pk, alice_msg.as_ref()).unwrap();

	//Bob decrypt it with his own private key
	let bob_decrypt = alg::asym::ecies::decrypt(&bob_secret, &alice_encrypted).unwrap();
	let bob_msg = std::str::from_utf8(&bob_decrypt).unwrap();

	assert_eq!(bob_msg, alice_msg);

	alice_msg.to_string() + " " + bob_msg
}

pub fn argon() -> String
{
	let master_key = alg::sym::aes_gcm::generate_key().unwrap();

	let key = match master_key.key {
		SymKey::Aes(k) => k,
	};

	let out = alg::pw_hash::argon2::derived_keys_from_password(b"abc", &key).unwrap();

	let encrypted_master_key = out.master_key_info.encrypted_master_key;

	Base64::encode_string(&encrypted_master_key)
}

pub fn argon_pw_encrypt() -> String
{
	let test = "plaintext message";

	//encrypt a value with a password, in prod this might be the key of the content
	let (aes_key_for_encrypt, salt) = alg::pw_hash::argon2::password_to_encrypt(b"my fancy password").unwrap();

	let encrypted = alg::sym::aes_gcm::encrypt_with_generated_key(&aes_key_for_encrypt, test.as_ref()).unwrap();

	//decrypt a value with password
	let aes_key_for_decrypt = alg::pw_hash::argon2::password_to_decrypt(b"my fancy password", &salt).unwrap();

	let decrypted = alg::sym::aes_gcm::decrypt_with_generated_key(&aes_key_for_decrypt, &encrypted).unwrap();

	let str = std::str::from_utf8(&decrypted).unwrap();

	assert_eq!(str, test);

	str.to_owned()
}

pub fn sign() -> String
{
	let test = "plaintext message";

	let out = alg::sign::ed25519::generate_key_pair();

	let out = match out {
		Err(_e) => return String::from("error"),
		Ok(o) => o,
	};

	let data_with_sig = alg::sign::ed25519::sign(&out.sign_key, test.as_bytes()).unwrap();

	let check = alg::sign::ed25519::verify(&out.verify_key, &data_with_sig).unwrap();

	assert_eq!(check, true);

	format!("check was: {}", check)
}

pub fn register_test() -> String
{
	let password = "abc*èéöäüê";

	let out = register_internally(password.to_string()).unwrap();

	//and now try to login
	//normally the salt gets calc by the api
	let client_random_value = match out.client_random_value {
		ClientRandomValue::Argon2(v) => v,
	};
	let salt_from_rand_value = alg::pw_hash::argon2::generate_salt(client_random_value);

	let prep_login_out = prepare_login_internally(password.to_string(), &salt_from_rand_value, out.derived_alg).unwrap();

	//try to decrypt the master key
	//prepare the encrypted values (from server in base64 encoded)

	let login_out = done_login_internally(
		&prep_login_out.master_key_encryption_key, //the value comes from prepare login
		&out.master_key_info.encrypted_master_key,
		&out.encrypted_private_key,
		out.keypair_encrypt_alg,
		&out.encrypted_sign_key,
		out.keypair_sign_alg,
	)
	.unwrap();

	//try encrypt / decrypt with the keypair
	let public_key = out.public_key;

	let text = "Hello world üöäéèßê°";
	let encrypted = alg::asym::ecies::encrypt(&public_key, text.as_bytes()).unwrap();
	let decrypted = alg::asym::ecies::decrypt(&login_out.private_key, &encrypted).unwrap();
	let decrypted_text = std::str::from_utf8(&decrypted).unwrap();

	//try sign and verify
	let verify_key = out.verify_key;

	let data_with_sign = alg::sign::ed25519::sign(&login_out.sign_key, &encrypted).unwrap();
	let verify_res = alg::sign::ed25519::verify(&verify_key, &data_with_sign).unwrap();

	format!("register sign result was: {} and decrypted text was: {}", verify_res, decrypted_text)
}

#[cfg(test)]
mod test
{
	use super::*;

	#[test]
	fn test_aes()
	{
		let str = aes();

		assert_eq!(str, "plaintext message plaintext message2");
	}

	#[test]
	fn test_ecdh()
	{
		let str = ecdh();

		assert_eq!(str, "Hello Bob Hello Bob");
	}

	#[test]
	fn test_register()
	{
		let str = argon();

		assert_ne!(str.len(), 0);
	}

	#[test]
	fn test_pw_encrypt()
	{
		let str = argon_pw_encrypt();

		assert_eq!(str, "plaintext message");
	}

	#[test]
	fn test_sign()
	{
		let str = sign();

		assert_eq!(str, "check was: true");
	}

	#[test]
	fn test_register_full()
	{
		let str = register_test();

		assert_eq!(str, "register sign result was: true and decrypted text was: Hello world üöäéèßê°");
	}
}
