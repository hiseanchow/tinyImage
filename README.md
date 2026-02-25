# TinyImage

基于 TinyPNG API 的跨平台图片压缩桌面应用，支持 macOS 和 Windows。

## 功能

- 拖拽图片或点击选择文件，批量压缩
- 支持 PNG、JPG、JPEG、WebP 格式
- 右键菜单集成（文件管理器中直接压缩）
- 压缩进度实时显示，节省空间一目了然

**设置项：**
| 选项 | 说明 |
|------|------|
| API Key | TinyPNG 开发者 Key（免费版每月 500 张） |
| 通知方式 | 弹窗 / 系统通知 / 静默 |
| 输出方式 | 原路径添加 `-tiny` 后缀 / 覆盖原图 / 指定目录 |

## 技术栈

- **前端**：Tauri 2 + Vue 3 + TypeScript + Vite
- **后端**：Rust（reqwest 调用 TinyPNG API，系统原生操作）

## 开发

```bash
# 安装依赖
pnpm install

# 开发模式（同时启动 Vite 和 Tauri）
pnpm run tauri dev

# 构建发布包
pnpm run tauri build
```

## 获取 API Key

访问 [tinypng.com/developers](https://tinypng.com/developers) 注册，免费版每月可压缩 500 张图片。

## 右键菜单

在设置页面启用"右键菜单集成"后：
- **macOS**：在 Finder 右键菜单的"服务"子菜单中出现"用 TinyImage 压缩"
- **Windows**：在资源管理器右键菜单中直接出现"用 TinyImage 压缩"
