use std::fs;
use std::path::PathBuf;

use base64::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binary {
    data: Vec<u8>,
}

impl Binary {
    /// 使用 Vec<U8> 构造二进制数据集
    pub fn build(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// 通过文件读取直接构造二进制数据集
    pub fn from_file(path: PathBuf) -> anyhow::Result<Self> {
        let data = fs::read(path)?;
        Ok(Self { data })
    }

    pub fn from_b64(value: String) -> anyhow::Result<Self> {
        let data = BASE64_STANDARD.decode(value)?;
        Ok(Self { data })
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn read(&self) -> Vec<u8> {
        self.data.clone()
    }
}

impl std::fmt::Display for Binary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "binary!({})", BASE64_STANDARD.encode(&self.data))
    }
}

