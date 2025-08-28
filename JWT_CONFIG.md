# JWT Token 配置说明

## 配置方式

CCline 现在支持从配置文件读取 JWT token，用于排名功能的 API 认证。

### 方式1：配置文件 (推荐)

1. 复制 `config.example.toml` 为 `config.toml`
2. 在配置文件中设置您的 JWT token：

```toml
# JWT Token 配置 (用于排名功能)
jwt_token = "your_jwt_token_here"
```

### 方式2：环境变量 (备用)

如果配置文件中没有设置 JWT token，系统会尝试从环境变量读取：

```bash
# Windows PowerShell
$env:PACKYCODE_JWT_TOKEN="your_jwt_token_here"

# Windows CMD
set PACKYCODE_JWT_TOKEN=your_jwt_token_here

# Linux/macOS
export PACKYCODE_JWT_TOKEN="your_jwt_token_here"
```

### 方式3：默认值 (fallback)

如果以上两种方式都没有配置，系统会使用代码中的默认 JWT token。

## 优先级

JWT token 的读取优先级为：
1. **配置文件** (`config.toml` 中的 `jwt_token`)
2. **环境变量** (`PACKYCODE_JWT_TOKEN`)
3. **默认值** (代码中硬编码的 token)

## 获取 JWT Token

1. 登录 PackyCode 网站 (https://share.packycode.com)
2. 打开浏览器开发者工具 (F12)
3. 在 Network 标签页中查找 API 请求
4. 复制请求头中的 `Authorization: Bearer` 后面的 token

## 配置文件位置

配置文件应该放在与 `statusline.exe` 相同的目录下，文件名为 `config.toml`。

### 创建配置文件

1. **复制示例配置**：
   ```bash
   # 在statusline.exe所在目录下
   copy config.example.toml config.toml
   ```

2. **编辑配置文件**：
   ```bash
   # 编辑同目录下的config.toml
   notepad config.toml
   ```

## 示例配置

参考 `config.example.toml` 文件中的完整配置示例。

## 注意事项

1. **Token 有效期**: JWT token 有过期时间，需要定期更新
2. **安全性**: 不要将包含真实 token 的配置文件提交到版本控制系统
3. **格式**: 确保 TOML 格式正确，字符串需要用双引号包围
