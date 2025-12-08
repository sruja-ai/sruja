import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { MermaidDiagram } from './MermaidDiagram'

export interface MarkdownPreviewProps {
  content: string
  onMermaidExpand?: (svg: string, code: string) => void
}

export function MarkdownPreview({ content, onMermaidExpand }: MarkdownPreviewProps) {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      components={{
        code: ({ inline, className, children, ...props }: any) => {
          const match = /language-(\w+)/.exec(className || '')
          const codeString = String(children || '').trim()
          if (!inline && match && match[1] === 'mermaid') {
            return <MermaidDiagram code={codeString} onExpand={onMermaidExpand} />
          }
          return (
            <code className={className} {...props}>
              {children}
            </code>
          )
        },
      }}
    >
      {content}
    </ReactMarkdown>
  )
}

