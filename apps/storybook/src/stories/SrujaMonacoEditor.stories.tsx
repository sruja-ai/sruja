// apps/storybook/src/stories/SrujaMonacoEditor.stories.tsx
import { SrujaMonacoEditor } from '../../../../packages/ui/src/components/SrujaMonacoEditor';
import type { Meta, StoryObj } from '@storybook/react';

const meta: Meta<typeof SrujaMonacoEditor> = {
  title: 'Components/SrujaMonacoEditor',
  component: SrujaMonacoEditor,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'A Monaco Editor component configured for Sruja language with optional LSP support. Provides syntax highlighting, autocomplete, and language features.',
      },
    },
  },
  argTypes: {
    value: {
      control: { type: 'text' },
      description: 'Initial editor content',
    },
    height: {
      control: { type: 'text' },
      description: 'Height of the editor',
      table: {
        defaultValue: { summary: '100%' },
      },
    },
    theme: {
      control: { type: 'select' },
      options: ['vs', 'vs-dark', 'hc-black'],
      description: 'Editor theme',
      table: {
        defaultValue: { summary: 'vs' },
      },
    },
    enableLsp: {
      control: { type: 'boolean' },
      description: 'Whether to enable LSP features',
      table: {
        defaultValue: { summary: 'true' },
      },
    },
  },
};

export default meta;
type Story = StoryObj<typeof SrujaMonacoEditor>;

const exampleSruja = `architecture "Example System" {
    system App "My Application" {
        container Web "Web Server" {
            component Auth "Authentication"
            component API "REST API"
        }
        datastore DB "Database"
    }
    
    person User "End User"
    
    User -> App.Web "Uses"
    App.Web.API -> App.DB "Reads/Writes"
}`;

export const Default: Story = {
  args: {
    value: exampleSruja,
    height: '400px',
  },
};

export const DarkTheme: Story = {
  args: {
    value: exampleSruja,
    height: '400px',
    theme: 'vs-dark',
  },
};

export const WithoutLsp: Story = {
  args: {
    value: exampleSruja,
    height: '400px',
    enableLsp: false,
  },
};

export const Small: Story = {
  args: {
    value: `architecture "Simple" {
    system App "Application"
}`,
    height: '200px',
  },
};

export const Large: Story = {
  args: {
    value: exampleSruja,
    height: '600px',
  },
};

