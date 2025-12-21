// apps/storybook/src/stories/MarkdownPreview.stories.tsx
import { MarkdownPreview } from '../../../../packages/ui/src/components/MarkdownPreview';
import type { Meta, StoryObj } from '@storybook/react';

const meta: Meta<typeof MarkdownPreview> = {
  title: 'Components/MarkdownPreview',
  component: MarkdownPreview,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'A markdown preview component that renders markdown content with support for GitHub Flavored Markdown and Mermaid diagrams.',
      },
    },
  },
  argTypes: {
    content: {
      control: { type: 'text' },
      description: 'Markdown content to render',
    },
  },
};

export default meta;
type Story = StoryObj<typeof MarkdownPreview>;

const basicMarkdown = `# Heading 1

This is a paragraph with **bold** and *italic* text.

## Heading 2

- List item 1
- List item 2
- List item 3

### Code Block

\`\`\`javascript
function hello() {
  // eslint-disable-next-line no-console
  console.log('Hello, world!');
}
\`\`\`
`;

export const Default: Story = {
  args: {
    content: basicMarkdown,
  },
};

export const WithMermaid: Story = {
  args: {
    content: `# Architecture Diagram

This is a Mermaid diagram:

\`\`\`mermaid
graph TD
    A[User] --> B[Web Server]
    B --> C[Database]
    B --> D[Cache]
\`\`\`

And some more text after the diagram.
`,
  },
};

export const GitHubFlavoredMarkdown: Story = {
  args: {
    content: `# GFM Features

## Tables

| Feature | Status |
|---------|--------|
| Tables  | ✅     |
| Strikethrough | ✅ |
| Task Lists | ✅ |

## Task Lists

- [x] Completed task
- [ ] Incomplete task
- [x] Another completed task

## Strikethrough

~~This text is strikethrough~~
`,
  },
};

export const LongContent: Story = {
  args: {
    content: `# Long Document

${Array(10).fill(0).map((_, i) => `## Section ${i + 1}

This is section ${i + 1} with some content. It demonstrates how the component handles longer documents.

- Item 1
- Item 2
- Item 3

\`\`\`typescript
const example = "This is code";
\`\`\`
`).join('\n\n')}
`,
  },
};

