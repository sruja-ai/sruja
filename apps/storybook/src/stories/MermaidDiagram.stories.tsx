// apps/storybook/src/stories/MermaidDiagram.stories.tsx
import { MermaidDiagram } from '../../../../packages/ui/src/components/MermaidDiagram';
import type { Meta, StoryObj } from '@storybook/react';

const meta: Meta<typeof MermaidDiagram> = {
  title: 'Components/MermaidDiagram',
  component: MermaidDiagram,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component: 'A component that renders Mermaid diagrams from code. Supports various diagram types including flowcharts, sequence diagrams, and more.',
      },
    },
  },
  argTypes: {
    code: {
      control: { type: 'text' },
      description: 'Mermaid diagram code',
    },
  },
};

export default meta;
type Story = StoryObj<typeof MermaidDiagram>;

export const Flowchart: Story = {
  args: {
    code: `graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Action 1]
    B -->|No| D[Action 2]
    C --> E[End]
    D --> E`,
  },
};

export const SequenceDiagram: Story = {
  args: {
    code: `sequenceDiagram
    participant User
    participant Web
    participant API
    participant DB
    
    User->>Web: Request
    Web->>API: API Call
    API->>DB: Query
    DB-->>API: Results
    API-->>Web: Response
    Web-->>User: Render`,
  },
};

export const ArchitectureDiagram: Story = {
  args: {
    code: `graph LR
    A[User] --> B[Web Server]
    B --> C[API Gateway]
    C --> D[Service 1]
    C --> E[Service 2]
    D --> F[Database]
    E --> F`,
  },
};

export const StateDiagram: Story = {
  args: {
    code: `stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: Start
    Processing --> Success: Complete
    Processing --> Error: Fail
    Success --> [*]
    Error --> Idle: Retry`,
  },
};

export const GanttChart: Story = {
  args: {
    code: `gantt
    title Project Timeline
    dateFormat YYYY-MM-DD
    section Phase 1
    Task 1 :a1, 2024-01-01, 30d
    Task 2 :a2, after a1, 20d
    section Phase 2
    Task 3 :a3, 2024-02-01, 25d`,
  },
};

export const ClassDiagram: Story = {
  args: {
    code: `classDiagram
    class User {
        +String name
        +String email
        +login()
    }
    class Order {
        +Date date
        +calculateTotal()
    }
    User "1" --> "*" Order`,
  },
};

