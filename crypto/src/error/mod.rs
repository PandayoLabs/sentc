use alloc::format;
use alloc::string::{String, ToString};

use sentc_crypto_core::Error;

#[derive(Debug)]
pub enum SdkError
{
	Base(Error),
	JsonToStringFailed,
	JsonParseFailed,

	DecodeSaltFailed,
	DecodeRandomValueFailed,
	DecodeHashedAuthKey,
	DecodeAuthKey,
	DecodeEncryptedDataFailed,
	DecodePublicKeyFailed,
	DecodeSymKeyFailed,

	ImportingSignKeyFailed,
	ImportingPrivateKeyFailed,
	ImportSymmetricKeyFailed,
	ImportPublicKeyFailed,
	ImportVerifyKeyFailed,
	ImportingKeyFromPemFailed,

	ExportingPublicKeyFailed,

	DerivedKeyWrongFormat,

	SigFoundNotKey,
	VerifyFailed,
	KeyDecryptFailed,

	LoginServerOutputWrong,
	KeyRotationServerOutputWrong,

	AlgNotFound,

	ServerErr(u32, String),
}

/**
To convert the core error to sdk error
*/
impl From<Error> for SdkError
{
	fn from(e: Error) -> Self
	{
		SdkError::Base(e)
	}
}

pub fn err_to_msg(error: SdkError) -> String
{
	match error {
		SdkError::Base(base_error) => {
			match base_error {
				Error::AlgNotFound => out_error("client_1", "The algorithms for this action was not found."),

				Error::DecodePrivateKeyFailed => out_error("client_3", "The private key has a wrong format."),
				_ => out_error("client_0", "other"),
			}
		},
		SdkError::AlgNotFound => out_error("client_1", "The algorithms for this action was not found."),
		SdkError::JsonToStringFailed => out_error("client_100", "Cannot create a string from this object"),
		SdkError::JsonParseFailed => out_error("client_101", "Cannot create an object from the input string"),
		//key decode error (from base64 string to the enum
		SdkError::DerivedKeyWrongFormat => out_error("client_2", "The encrypted key has a wrong format."),
		//salt decode error (from base64 string to bytes)
		SdkError::DecodeSaltFailed => out_error("client_4", "The salt has a wrong format"),
		SdkError::ServerErr(code, msg) => {
			out_error(
				(String::from("server_") + code.to_string().as_str()).as_str(),
				msg.as_str(),
			)
		},
		_ => out_error("client_0", "other"),
	}
}

pub(crate) fn out_error(code: &str, message: &str) -> String
{
	//create the error in json to communicate with the other implementations, so they can use their own error handling

	format!("{{\"status\": {}, \"error_message\": \"{}\"}}", code, message)
}
