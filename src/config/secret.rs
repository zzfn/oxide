//! 敏感信息保护模块
//!
//! 提供 Secret 类型用于包装敏感信息（如 API Token、密码等），
//! 防止其在日志、错误消息或调试输出中意外泄露。

use std::fmt;
use std::ops::Deref;
use zeroize::Zeroize;

/// 包装敏感信息的类型
///
/// 该类型通过实现自定义的 Debug 和 Display trait 来防止敏感信息泄露，
/// 并支持使用 `zeroize()` 方法安全清除内存中的数据。
///
/// # 示例
///
/// ```
/// use oxide::config::secret::Secret;
///
/// let token = Secret::new("sk-ant-api03-...".to_string());
/// println!("{:?}", token); // 输出: "***"
/// println!("{}", token);    // 输出: "***"
///
/// // 使用内部值
/// let token_str = token.expose_secret(); // "sk-ant-api03-..."
/// assert_eq!(token_str, "sk-ant-api03-...");
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// 创建新的 Secret 实例
    ///
    /// # 参数
    ///
    /// * `value` - 需要保护的敏感值
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// 获取内部值的引用
    ///
    /// # 警告
    ///
    /// 此方法会暴露受保护的值，请谨慎使用。仅在必须使用原始值的地方调用
    ///（如发送 API 请求）。
    ///
    /// # 示例
    ///
    /// ```
    /// use oxide::config::secret::Secret;
    ///
    /// let token = Secret::new("my-secret-token".to_string());
    /// // 使用暴露的值
    /// let token_str = token.expose_secret();
    /// assert_eq!(token_str, "my-secret-token");
    /// ```
    pub fn expose_secret(&self) -> &T {
        &self.0
    }

    /// 获取内部值的可变引用
    ///
    /// # 警告
    ///
    /// 此方法会暴露受保护的值，请谨慎使用。
    pub fn expose_secret_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// 消费 Self 并返回内部值
    ///
    /// # 警告
    ///
    /// 此方法会转移所有权并暴露受保护的值，请谨慎使用。
    pub fn into_inner(self) -> T {
        self.0
    }
}

// 为 Secret<String> 实现安全的内存清除
impl Zeroize for Secret<String> {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

// 防止敏感信息在 Debug 输出中泄露
impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

// 防止敏感信息在 Display 输出中泄露
impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

// 实现 Deref 以便方便访问内部值（但不暴露）
impl<T> Deref for Secret<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// 为常见类型实现便捷构造函数
impl Secret<String> {
    /// 从 &str 创建 Secret<String>
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_hides_in_debug() {
        let secret = Secret::new("my-secret-value".to_string());
        let debug_output = format!("{:?}", secret);
        assert_eq!(debug_output, "***");
    }

    #[test]
    fn test_secret_hides_in_display() {
        let secret = Secret::new("my-secret-value".to_string());
        let display_output = format!("{}", secret);
        assert_eq!(display_output, "***");
    }

    #[test]
    fn test_expose_secret() {
        let secret = Secret::new("my-secret-value".to_string());
        assert_eq!(secret.expose_secret(), "my-secret-value");
    }

    #[test]
    fn test_expose_secret_mut() {
        let mut secret = Secret::new("my-secret-value".to_string());
        *secret.expose_secret_mut() = "new-value".to_string();
        assert_eq!(secret.expose_secret(), "new-value");
    }

    #[test]
    fn test_into_inner() {
        let secret = Secret::new("my-secret-value".to_string());
        let inner = secret.into_inner();
        assert_eq!(inner, "my-secret-value");
    }

    #[test]
    fn test_from_str() {
        let secret = Secret::from_str("test-value");
        assert_eq!(secret.expose_secret(), "test-value");
    }

    #[test]
    fn test_equality() {
        let secret1 = Secret::new("value".to_string());
        let secret2 = Secret::new("value".to_string());
        let secret3 = Secret::new("other".to_string());

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
    }

    #[test]
    fn test_clone() {
        let secret = Secret::new("value".to_string());
        let cloned = secret.clone();
        assert_eq!(secret, cloned);
    }

    #[test]
    fn test_zeroize() {
        let mut secret = Secret::new("sensitive-data".to_string());
        secret.zeroize();
        assert_eq!(secret.expose_secret(), "");
    }
}
