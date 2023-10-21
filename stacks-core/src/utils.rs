use std::io;

use strum::FromRepr;

use crate::{
	address::{AddressVersion, StacksAddress},
	codec::Codec,
	contract_name::ContractName,
	StacksError,
};

#[derive(PartialEq, Eq, Debug, Clone)]
/// Standard principal data type
pub struct StandardPrincipalData(pub AddressVersion, pub StacksAddress);

impl StandardPrincipalData {
	/// Create a new standard principal data type from the provided address
	/// version and stacks address
	pub fn new(version: AddressVersion, address: StacksAddress) -> Self {
		Self(version, address)
	}
}

impl Codec for StandardPrincipalData {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		self.1.codec_serialize(dest)
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let addr = StacksAddress::codec_deserialize(data)?;

		Ok(Self(addr.version(), addr))
	}
}

impl From<StacksAddress> for StandardPrincipalData {
	fn from(address: StacksAddress) -> Self {
		Self(address.version(), address)
	}
}

#[derive(PartialEq, Eq, Debug, Clone)]
/// Principal Data type
pub enum PrincipalData {
	/// Standard principal data type
	Standard(StandardPrincipalData),
	/// Contract principal data type
	Contract(StandardPrincipalData, ContractName),
}

#[repr(u8)]
#[derive(FromRepr, Debug, Clone, Copy)]
enum PrincipalTypeByte {
	Standard = 0x05,
	Contract = 0x06,
}

impl Codec for PrincipalData {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		match self {
			Self::Standard(data) => {
				dest.write_all(&[PrincipalTypeByte::Standard as u8])?;
				data.codec_serialize(dest)
			}
			Self::Contract(data, contract_name) => {
				dest.write_all(&[PrincipalTypeByte::Contract as u8])?;
				data.codec_serialize(dest)?;
				contract_name.codec_serialize(dest)
			}
		}
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let mut type_buffer = [0u8; 1];
		data.read_exact(&mut type_buffer)?;

		let principal_type = PrincipalTypeByte::from_repr(type_buffer[0])
			.ok_or_else(|| {
				io::Error::new(
					io::ErrorKind::InvalidData,
					format!("Invalid principal type: {}", type_buffer[0]),
				)
			})?;

		match principal_type {
			PrincipalTypeByte::Standard => {
				let standard_data =
					StandardPrincipalData::codec_deserialize(data)?;

				Ok(Self::Standard(standard_data))
			}
			PrincipalTypeByte::Contract => {
				let standard_data =
					StandardPrincipalData::codec_deserialize(data)?;
				let contract_name = ContractName::codec_deserialize(data)?;

				Ok(Self::Contract(standard_data, contract_name))
			}
		}
	}
}

impl From<StacksAddress> for PrincipalData {
	fn from(address: StacksAddress) -> Self {
		Self::Standard(StandardPrincipalData::from(address))
	}
}

impl TryFrom<String> for PrincipalData {
	type Error = StacksError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let parts: Vec<&str> = value.split('.').collect();

		match parts.len() {
			1 => {
				let address = StacksAddress::try_from(parts[0])?;
				Ok(Self::Standard(StandardPrincipalData::from(address)))
			}
			2 => {
				let address = StacksAddress::try_from(parts[0])?;
				let contract_name =
					ContractName::new(parts[1]).map_err(|err| {
						StacksError::InvalidData(format!(
							"Invalid contract name from {value}: {err}"
						))
					})?;

				Ok(Self::Contract(
					StandardPrincipalData::from(address),
					contract_name,
				))
			}
			_ => Err(StacksError::InvalidData(format!(
				"Principal data from {value} may contain at most 1 dot character"
			))),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::crypto::hash160::Hash160Hasher;

	#[test]
	fn should_serialize_standard_principal_data() {
		let addr = StacksAddress::new(
			AddressVersion::TestnetSingleSig,
			Hash160Hasher::default(),
		);
		let data = PrincipalData::Standard(StandardPrincipalData(
			addr.version(),
			addr.clone(),
		));

		let mut expected_bytes = vec![];

		expected_bytes.push(PrincipalTypeByte::Standard as u8);
		expected_bytes.push(addr.version() as u8);
		expected_bytes.extend(addr.hash().as_ref());

		let serialized = data.serialize_to_vec();

		assert_eq!(serialized, expected_bytes);
	}

	#[test]
	fn should_deserialize_standard_principal_data() {
		let addr = StacksAddress::new(
			AddressVersion::TestnetSingleSig,
			Hash160Hasher::default(),
		);
		let expected_principal_data = PrincipalData::Standard(
			StandardPrincipalData(addr.version(), addr.clone()),
		);

		let mut expected_bytes = vec![];

		expected_bytes.push(PrincipalTypeByte::Standard as u8);
		expected_bytes.push(addr.version() as u8);
		expected_bytes.extend(addr.hash().as_ref());

		let serialized = expected_principal_data.serialize_to_vec();
		let deserialized =
			PrincipalData::deserialize(&mut &serialized[..]).unwrap();

		assert_eq!(deserialized, expected_principal_data);
	}

	#[test]
	fn should_serialize_contract_principal_data() {
		let addr = StacksAddress::new(
			AddressVersion::TestnetSingleSig,
			Hash160Hasher::default(),
		);
		let contract = ContractName::new("helloworld").unwrap();
		let data = PrincipalData::Contract(
			StandardPrincipalData(addr.version(), addr.clone()),
			contract.clone(),
		);

		let mut expected_bytes = vec![];

		expected_bytes.push(PrincipalTypeByte::Contract as u8);
		expected_bytes.push(addr.version() as u8);
		expected_bytes.extend(addr.hash().as_ref());
		expected_bytes.push(contract.len() as u8);
		expected_bytes.extend(contract.as_bytes());

		let serialized = data.serialize_to_vec();

		assert_eq!(serialized, expected_bytes);
	}

	#[test]
	fn should_deserialize_contract_principal_data() {
		let addr = StacksAddress::new(
			AddressVersion::TestnetSingleSig,
			Hash160Hasher::default(),
		);
		let contract = ContractName::new("helloworld").unwrap();
		let expected_principal_data = PrincipalData::Contract(
			StandardPrincipalData(addr.version(), addr.clone()),
			contract.clone(),
		);

		let mut expected_bytes = vec![];

		expected_bytes.push(PrincipalTypeByte::Contract as u8);
		expected_bytes.push(addr.version() as u8);
		expected_bytes.extend(addr.hash().as_ref());
		expected_bytes.push(contract.len() as u8);
		expected_bytes.extend(contract.as_bytes());

		let serialized = expected_principal_data.serialize_to_vec();
		let deserialized =
			PrincipalData::deserialize(&mut &serialized[..]).unwrap();

		assert_eq!(deserialized, expected_principal_data);
	}

	#[test]
	fn should_principal_data_try_from_string() {
		// addr = ST000000000000000000002AMW42H
		let addr = StacksAddress::new(
			AddressVersion::TestnetSingleSig,
			Hash160Hasher::default(),
		);
		let principal_data: PrincipalData = PrincipalData::try_from(
			"ST000000000000000000002AMW42H.helloworld".to_string(),
		)
		.unwrap();

		assert_eq!(
			principal_data,
			PrincipalData::Contract(
				StandardPrincipalData(addr.version(), addr.clone()),
				ContractName::new("helloworld").unwrap(),
			)
		);
	}

	#[test]
	fn should_fail_to_convert_invalid_string_to_principal_data() {
		// try invalid address
		let mut result =
			PrincipalData::try_from("ST123.helloworld".to_string());

		assert_eq!(
			result.unwrap_err().to_string(),
			StacksError::C32Error(crate::c32::C32Error::InvalidAddress(
				"ST123".into()
			))
			.to_string()
		);

		// try contract name with a space
		result = PrincipalData::try_from(
			"ST000000000000000000002AMW42H.hello contract".to_string(),
		);

		assert_eq!(
			result.unwrap_err().to_string(),
			StacksError::InvalidData("Invalid contract name from ST000000000000000000002AMW42H.hello contract: Format should follow the contract name specification".into()).to_string()
		);
	}
}
