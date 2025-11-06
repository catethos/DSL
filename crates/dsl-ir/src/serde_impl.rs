use crate::ir::IR;
use anyhow::{Context, Result};

impl IR {
    /// Serialize to MessagePack binary format
    pub fn to_msgpack(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self).context("Failed to serialize IR to MessagePack")
    }

    /// Deserialize from MessagePack binary format
    pub fn from_msgpack(bytes: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(bytes).context("Failed to deserialize IR from MessagePack")
    }

    /// Serialize to JSON (for debugging/inspection)
    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize IR to JSON")
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to deserialize IR from JSON")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::IRNode;

    #[test]
    fn test_ir_msgpack_roundtrip() {
        let ir = IR {
            version: "0.1.0".to_string(),
            types: vec![],
            enums: vec![],
            functions: vec![],
            agents: vec![],
            entry_expr: IRNode::Int(42),
        };

        let bytes = ir.to_msgpack().unwrap();
        let restored = IR::from_msgpack(&bytes).unwrap();

        assert_eq!(ir.version, restored.version);
        assert_eq!(ir.types.len(), restored.types.len());
        assert_eq!(ir.enums.len(), restored.enums.len());
        assert_eq!(ir.functions.len(), restored.functions.len());
    }

    #[test]
    fn test_ir_json_roundtrip() {
        let ir = IR {
            version: "0.1.0".to_string(),
            types: vec![],
            enums: vec![],
            functions: vec![],
            agents: vec![],
            entry_expr: IRNode::String("hello".to_string()),
        };

        let json = ir.to_json_pretty().unwrap();
        let restored = IR::from_json(&json).unwrap();

        assert_eq!(ir.version, restored.version);
        match restored.entry_expr {
            IRNode::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string node"),
        }
    }
}
