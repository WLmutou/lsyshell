

// 新增自定义错误类型
#[derive(Debug)]
pub enum LsyError {
    Io(std::io::Error),
    Ssh(ssh2::Error),
    JsonError(serde_json::Error),
}

impl std::fmt::Display for LsyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LsyError::Io(e) => write!(f, "IO错误: {}", e),
            LsyError::Ssh(e) => write!(f, "SSH错误: {}", e),
            LsyError::JsonError(e) => write!(f, "json结构错误: {}", e),
        }
    }
}

impl std::error::Error for LsyError {}

// 实现错误类型转换
impl From<std::io::Error> for LsyError {
    fn from(err: std::io::Error) -> Self {
        LsyError::Io(err)
    }
}

impl From<ssh2::Error> for LsyError {
    fn from(err: ssh2::Error) -> Self {
        LsyError::Ssh(err)
    }
}




impl From<serde_json::Error> for LsyError {
    fn from(err: serde_json::Error) -> Self {
        LsyError::JsonError(err)
    }
}