import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { MermaidDiagram } from './MermaidDiagram'
import './MarkdownPreview.css'

export interface MarkdownPreviewProps {
  content: string
  onMermaidExpand?: (svg: string, code: string) => void
  className?: string
}

export function MarkdownPreview({ content, onMermaidExpand, className = '' }: MarkdownPreviewProps) {
  return (
    <div className={`markdown-preview ${className}`}>
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
    </div>
  )
}

