use std::path::PathBuf;
use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binary {
    data: Vec<u8>
}

impl Binary {
    
    /// 使用 Vec<U8> 构造二进制数据集
    pub fn build(data: Vec<u8>) -> Self {
        Self {
            data
        }
    }

    /// 通过文件读取直接构造二进制数据集
    pub fn from_file(path: PathBuf) -> anyhow::Result<Self> {
        let data = fs::read(path)?;
        return Ok (
            Self {
                data
            }
        );
    }

    pub fn from_b64(value: String) -> anyhow::Result<Self> {
        let data = base64::decode(value)?;
        return Ok (
            Self {
                data
            }
        );
    }

    pub fn size(&self) -> usize {
        return self.data.len();
    }

}

impl ToString for Binary {
    fn to_string(&self) -> String {
        format!("binary!({})",base64::encode(self.data.clone()))
    }
}