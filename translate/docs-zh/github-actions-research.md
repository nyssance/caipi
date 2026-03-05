# Tauri macOS 构建的 GitHub Actions 研究

## 概述

本文档概述了将 Caipi 构建过程移至 GitHub Actions 的方法，为 Linux 和 Windows 的未来跨平台构建铺平道路。

## 架构

```
┌─────────────────────────────────┐      ┌─────────────────────────────┐
│  pietz/caipi (private)          │      │  pietz/caipi.ai (public)    │
│                                 │      │                             │
│  - Source code                  │──────│  - GitHub Release           │
│  - GitHub Actions workflow      │      │  - Casks/caipi.rb update    │
│  - Secrets for signing          │      │                             │
└─────────────────────────────────┘      └─────────────────────────────┘
```

## 所需的 GitHub Secrets

在 `pietz/caipi` → Settings → Secrets → Actions 中设置：

| Secret | 描述 |
|--------|------|
| `APPLE_CERTIFICATE` | Base64 编码的 `.p12` 文件 |
| `APPLE_CERTIFICATE_PASSWORD` | .p12 的密码 |
| `APPLE_ID` | 您的 Apple Developer 电子邮件 |
| `APPLE_PASSWORD` | 应用程序专用密码（不是您的帐户密码）|
| `APPLE_TEAM_ID` | 在 Apple Developer 会员资格页面中找到 |
| `KEYCHAIN_PASSWORD` | 任何密码（用于 CI 中的临时钥匙串）|
| `TAURI_SIGNING_PRIVATE_KEY` | 用于更新程序签名 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 签名密钥的密码 |
| `CAIPI_AI_PAT` | 具有对 pietz/caipi.ai 写入访问权限的个人访问令牌 |

要获取证书的 base64：
```bash
openssl base64 -in MyCertificate.p12 -out MyCertificate-base64.txt
```

## 工作流模板

```yaml
# .github/workflows/release.yml
name: Release

on:
  workflow_dispatch:
    inputs:
      version:
        description: '要发布的版本（例如，0.1.8）'
        required: true

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: lts/*
          cache: npm

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-darwin

      - uses: swatinem/rust-cache@v2
        with:
          workspaces: ./src-tauri -> target

      - run: npm ci

      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
          KEYCHAIN_PASSWORD: ${{ secrets.KEYCHAIN_PASSWORD }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          args: --target aarch64-apple-darwin

      - name: 重命名工件
        run: node scripts/release-rename.js

      - name: 在公共仓库上创建发布
        uses: softprops/action-gh-release@v1
        with:
          repository: pietz/caipi.ai
          token: ${{ secrets.CAIPI_AI_PAT }}
          tag_name: v${{ inputs.version }}
          files: |
            src-tauri/target/release/bundle/dmg/caipi_aarch64.dmg
            src-tauri/target/release/bundle/macos/caipi.app.tar.gz
            src-tauri/target/release/bundle/macos/caipi.app.tar.gz.sig
```

## 跨仓库发布选项

### 选项 1：softprops/action-gh-release
- 指定 `repository:` 参数以发布到不同的仓库
- 需要具有 `repo` 范围的 PAT
- 最适合创建带有工件的发布

### 选项 2：cpina/github-action-push-to-another-repository
- 用于将文件/提交推送到另一个仓库
- 对于更新 Homebrew cask formula 很有用

## 关键注意事项

1. **公证需要时间** - Apple 公证可能需要几分钟。当提供凭据时，`tauri-action` 会自动处理此问题。

2. **免费的 Apple Developer 帐户不起作用** - 公证需要付费的 Apple Developer Program 会员资格（99 美元/年）。

3. **Cask formula 更新** - 需要第二步来更新 `Casks/caipi.rb` 以及新的 SHA256 和版本，然后推送到公共仓库。

4. **通用二进制文件** - 要支持 Intel Mac，添加第二个矩阵条目，使用 `--target x86_64-apple-darwin`。

## 未来：多平台构建

```yaml
strategy:
  matrix:
    include:
      - platform: macos-latest
        args: --target aarch64-apple-darwin
      - platform: macos-latest
        args: --target x86_64-apple-darwin
      - platform: ubuntu-22.04
        args: ''
      - platform: windows-latest
        args: ''
```

Linux 需要额外的依赖：
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

## 参考资料

- [Tauri GitHub Pipelines 文档](https://v2.tauri.app/distribute/pipelines/github/)
- [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action)
- [Tauri macOS 代码签名](https://v2.tauri.app/distribute/sign/macos/)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)
- [跨仓库工件发布指南](https://dev.to/oysterd3/how-to-release-built-artifacts-from-one-to-another-repo-on-github-3oo5)
