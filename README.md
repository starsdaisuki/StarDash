# Tauri + Vue + TypeScript

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)


TAURI-TEST/
├── src/                ← 前端代码（Vue），你主要改这里
│   ├── App.vue         ← 主组件，现在显示的那个欢迎页面
│   ├── main.ts         ← 前端入口
│   └── ...
├── src-tauri/          ← Rust 后端，Tauri 的核心
│   ├── src/
│   │   └── lib.rs      ← 你写 Rust 命令的地方（读CPU等）
│   ├── Cargo.toml      ← Rust 的依赖配置（类似 package.json）
│   └── tauri.conf.json ← Tauri 配置（窗口大小、标题等）
├── index.html          ← 入口 HTML
├── package.json        ← 前端依赖配置
├── pnpm-lock.yaml      ← pnpm 的锁文件，不用管
├── vite.config.ts      ← Vite 配置
└── tsconfig.json       ← TypeScript 配置