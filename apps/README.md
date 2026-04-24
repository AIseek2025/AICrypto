# apps/

产品交互层，包含 Web 控制台、管理 API 和运维工具。

## 目录

| 目录 | 用途 | 技术栈 |
|------|------|--------|
| `console-web/` | 运营控制台前端 | Next.js + React + TypeScript + Tailwind |
| `admin-api/` | 后台管理接口 | TypeScript (Node.js) |
| `ops-tools/` | 运维辅助工具 | TypeScript |

## 边界

- 允许：UI 页面、管理接口、工具脚本
- 禁止：直接访问交易所 API、直接操作数据库
