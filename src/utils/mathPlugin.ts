import type { BytemdPlugin } from 'bytemd'
import type { BytemdPluginMathOptions } from '@bytemd/plugin-math'
import remarkMath from 'remark-math'

/**
 * 包装 @bytemd/plugin-math：关闭单美元行内公式。
 * `$` 在标准 Markdown 中是普通字符；启用单 `$...$` 会把金额（如 $412）误解析为公式。
 * 行内公式用 \(...\)，块级仍用 $$...$$。
 */
export const createMathPlugin = async (
  options?: BytemdPluginMathOptions
): Promise<BytemdPlugin> => {
  const { default: math } = await import('@bytemd/plugin-math')
  const base = math(options)

  return {
    ...base,
    // 覆盖 remark：关闭单 $ 解析，其余（KaTeX 渲染等）沿用原插件
    remark: (processor) => processor.use(remarkMath, { singleDollarTextMath: false }),
    actions: patchInlineMathActions(base.actions),
  }
}

/** 把工具栏「行内公式」从 $...$ 改为 \(...\) */
const patchInlineMathActions = (actions: BytemdPlugin['actions']): BytemdPlugin['actions'] => {
  if (!actions?.length) return actions

  return actions.map((action) => {
    const handler = action.handler
    if (!handler || handler.type !== 'dropdown' || !handler.actions) {
      return action
    }

    return {
      ...action,
      handler: {
        ...handler,
        actions: handler.actions.map((item) => {
          // 原插件 cheatsheet 为 `$formula$`
          if (typeof item.cheatsheet === 'string' && item.cheatsheet.startsWith('$') && !item.cheatsheet.startsWith('$$')) {
            return {
              ...item,
              cheatsheet: item.cheatsheet.replace(/^\$/, '\\(').replace(/\$$/, '\\)'),
              handler:
                item.handler?.type === 'action'
                  ? {
                      ...item.handler,
                      click(ctx) {
                        ctx.wrapText('\\(', '\\)')
                        ctx.editor.focus()
                      },
                    }
                  : item.handler,
            }
          }
          return item
        }),
      },
    }
  })
}
