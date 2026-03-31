# todo list

> 副标题：把任务变成可执行时间块

基于 **Tauri 2 + React + TypeScript + SQLite** 的本地优先桌面应用，面向研究生单用户场景，强调“执行闭环”而非纯记录。

---

## 一、怎么运行（开发模式）

> 以下命令在仓库根目录执行：`/workspace/liproxy`

### 1) 安装依赖

```bash
npm install
```

### 2) 启动桌面应用（Tauri + React）

```bash
npm run tauri:dev
```

这会同时：
- 启动 Vite 前端开发服务（默认 `http://localhost:1420`）
- 启动 Tauri 桌面窗口

### 3) 仅调试前端（不打开 Tauri 窗口）

```bash
npm run dev
```

---

## 二、怎么打包

### 打包前端资源

```bash
npm run build
```

### 打包桌面安装包

```bash
npm run tauri:build
```

产物通常在：

- `src-tauri/target/release/bundle/`

---

## 三、怎么使用（V1）

### 1) Inbox 快速收集

在 **Inbox** 页输入任务文本，按回车或点击创建，例如：

- `4月10日前交图论作业 预计6小时`
- `明天上午复习英语阅读 45分钟`
- `每周读一篇英文论文`

系统会尽量解析：
- 截止日期（如 `4月10日` / `明天`）
- 时长（如 `6小时` / `45分钟`）
- 类型（作业/英语/论文/训练）

解析失败也会保留原文，不会丢任务。

### 2) 任务拆分

对大任务点击“拆分时间块”：

- ≤60 分钟：不拆
- 61~180 分钟：拆 2 块
- >180 分钟：按 90~120 分钟分块

### 3) Today 今日执行

打开 **Today** 可看：
- 今日课程
- 今日必须做
- 今日可推进

### 4) Week 周视图（简版）

在 **Week** 可以：
- 查看课程
- 查看任务候选
- 新增课程（用于后续排期避让）

### 5) Focus 专注

在 **Focus**：
1. 选择任务
2. 选择 25 或 50 分钟
3. 点击开始

结束后会自动累计到任务投入时长。

### 6) Habits 习惯打卡

在 **Habits**：
- 新建习惯（健身/英语/论文阅读等）
- 点击“打卡”记录本周完成次数

### 7) Review 周复盘

在 **Review**：
- 查看本周完成任务数
- 总专注时长
- 英语/训练/论文阅读完成次数
- 系统给出的简要结论

---

## 四、数据库与数据位置

应用使用本地 SQLite，数据库文件默认在运行目录：

- `todo_timeblocks.db`

核心表：
- `tasks`
- `task_blocks`
- `courses`
- `focus_sessions`
- `habits`
- `habit_logs`
- `settings`

---

## 五、常见问题

### 1) `npm install` 报 403

通常是网络策略或镜像源限制，建议：
- 切换可用 npm registry
- 或在允许外网的环境下安装依赖

### 2) `cargo check` 拉取 crates 失败

通常是 Rust 依赖源被限制。建议：
- 配置可访问的 crates 镜像
- 或在有外网权限环境执行构建

---

## 六、当前实现范围说明

V1 已覆盖：
- Inbox 快速录入
- 任务管理核心动作
- 任务拆分
- Today / Week（简版）
- Focus
- Habits
- Review
- 本地持久化

后续迭代建议：
- Week 拖拽排期与冲突检测
- 自动排程建议可视化确认
- Settings 完整配置编辑
- 任务详情完整 CRUD
- 课程批量导入
