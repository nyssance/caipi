# 测试

此项目为前端（TypeScript/Svelte）和后端（Rust）设置了测试基础设施。

## 运行测试

```bash
# 运行所有测试（前端 + 后端）
npm run test:all

# 仅运行前端测试
npm test

# 以监视模式运行前端测试
npm run test:watch

# 仅运行后端测试
cd src-tauri && cargo test
```

## 共享固件（前端）

可重用的固件位于 `src/tests/fixtures` 下：
- `license.ts` 用于许可证状态
- `startup.ts` 用于启动信息负载
- `history.ts` 用于会话历史负载

## 前端测试

- **框架**：Vitest 搭配 @testing-library/svelte
- **环境**：happy-dom
- **位置**：测试与源文件位于同一位置（`*.test.ts` 文件）

### 示例：存储测试

请参阅 `src/lib/stores/chat.test.ts` 以获取测试 Svelte 存储的示例。

### 组件测试

已配置 Svelte 5 组件测试，但可能需要额外的设置才能与测试库完全兼容。对于大多数用例，建议进行存储测试。

## 后端测试

- **框架**：内置 Rust 测试框架
- **位置**：测试模块在实现文件的末尾用 `#[cfg(test)]` 定义

### 示例：单元测试

请参阅：
- `src-tauri/src/commands/chat.rs` - 会话管理测试
- `src-tauri/src/claude/permissions.rs` - 权限逻辑测试

## 当前测试覆盖

### 前端
- ✅ 聊天存储：消息管理、流式传输、活动、队列
- 📝 可以按照 chat.test.ts 模式添加额外的存储测试

### 后端
- ✅ 会话创建和权限的占位符测试
- 📝 测试可以扩展以涵盖实际实现逻辑

## 添加新测试

### 前端存储测试

在您的存储文件旁边创建一个 `<store-name>.test.ts` 文件：

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { myStore } from './my-store';
import { get } from 'svelte/store';

describe('myStore', () => {
  beforeEach(() => {
    myStore.reset();
  });

  it('应该做某事', () => {
    myStore.doSomething();
    expect(get(myStore).value).toBe('expected');
  });
});
```

### Rust 测试

在您的 Rust 文件末尾添加一个测试模块：

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        assert_eq!(2 + 2, 4);
    }
}
```
