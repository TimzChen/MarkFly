export interface FileItem {
  name: string
  content: string
  path: string
}

export const sampleFiles: FileItem[] = [
  {
    name: 'welcome.md',
    path: 'welcome.md',
    content: `# 🚀 欢迎使用 MarkFly

这是一个功能强大的 Markdown 编辑器，为您提供出色的写作体验。

## ✨ 主要特性

- **实时预览**: 边写边看，所见即所得
- **语法高亮**: 支持多种编程语言的代码高亮
- **数学公式**: 完整的 LaTeX 数学公式支持
- **图表支持**: 内置 Mermaid 流程图和图表
- **主题切换**: 支持亮色和暗色主题
- **跨平台**: 支持 Windows、macOS 和 Linux

## 🎯 快速开始

### 基本语法

使用 Markdown 语法来格式化您的文档：

- **粗体文本**: \`**粗体**\`
- *斜体文本*: \`*斜体*\`
- [链接](https://example.com): \`[链接文本](URL)\`
- \`行内代码\`: \`\\\`行内代码\\\`\`

### 代码块

\`\`\`javascript
function hello() {
    console.log('Hello, MarkFly!');
}
\`\`\`

### 数学公式

行内公式：\\(E = mc^2\\)

块级公式：
$$
\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \\sqrt{\\pi}
$$

### Mermaid 图表

\`\`\`mermaid
graph TD
    A[开始] --> B{条件判断}
    B -->|是| C[执行操作]
    B -->|否| D[结束]
    C --> D
\`\`\`

---

🎉 **开始您的创作之旅吧！**`
  }
]