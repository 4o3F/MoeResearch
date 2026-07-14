[根目录](../../CLAUDE.md) > [crates](../) > **moe-research-web-fetch**

# moe-research-web-fetch 模块文档

## 模块职责

该 crate 实现 Layer 2 内部 `web_fetch` 领域逻辑：URL 规范化、重定向、文档转换、并发安全的有界缓存、同 URL miss 合并、独立模型 prompt-processing 与 typed outcome。

## 边界

- 所有页面与模型 HTTP 必须通过注入的 `moe-research-net::NetworkClient`。
- 不直接依赖或创建 reqwest client。
- 不依赖 workflow、MCP、CLI 或 config。
- 不处理浏览器渲染、登录态、PDF/OCR、图片、音视频或任意请求头。
- 文档缓存使用 `RwLock` 支持并发命中；同一 normalized URL 的并发 miss 通过 per-key async singleflight 合并，锁不跨不同 URL 的网络请求。

## 测试

回归测试统一放在 `crates/moe-research-tests`，生产源码不新增 `#[cfg(test)]`。
