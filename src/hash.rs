use crate::{Hash, SerializeLittleEndian};

pub(crate) fn double_sha256(obj: &dyn SerializeLittleEndian) -> Hash {
    let mut serialized = Vec::new();
    obj.serialize_le(&mut serialized);
    let first_hash = hmac_sha256::Hash::hash(&serialized);
    Hash(hmac_sha256::Hash::hash(&first_hash)).reverse()
}
