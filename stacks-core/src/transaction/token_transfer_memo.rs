pub struct TokenTransferMemo([u8; 34]);

impl TokenTransferMemo {
    pub fn from_bytes(memo: impl AsRef<[u8]>) -> StacksResult<Self> {
        let bytes: [u8; 34] = memo.as_ref().try_into()?;

        Ok(bytes.into())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

impl From<[u8; 34]> for TokenTransferMemo {
    fn from(value: [u8; 34]) -> Self {
        Self(value)
    }
}

impl AsRef<[u8]> for TokenTransferMemo {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
