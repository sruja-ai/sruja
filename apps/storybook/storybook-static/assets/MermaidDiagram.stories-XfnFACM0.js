import{M as c}from"./MermaidDiagram-CE231KiM.js";import"./preload-helper-PPVm8Dsz.js";import"./iframe-CLHqt8sP.js";import"./logger-CfJbYVt4.js";const p={title:"Components/MermaidDiagram",component:c,tags:["autodocs"],parameters:{docs:{description:{component:"A component that renders Mermaid diagrams from code. Supports various diagram types including flowcharts, sequence diagrams, and more."}}},argTypes:{code:{control:{type:"text"},description:"Mermaid diagram code"}}},e={args:{code:`graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Action 1]
    B -->|No| D[Action 2]
    C --> E[End]
    D --> E`}},a={args:{code:`sequenceDiagram
    participant User
    participant Web
    participant API
    participant DB
    
    User->>Web: Request
    Web->>API: API Call
    API->>DB: Query
    DB-->>API: Results
    API-->>Web: Response
    Web-->>User: Render`}},r={args:{code:`graph LR
    A[User] --> B[Web Server]
    B --> C[API Gateway]
    C --> D[Service 1]
    C --> E[Service 2]
    D --> F[Database]
    E --> F`}},s={args:{code:`stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: Start
    Processing --> Success: Complete
    Processing --> Error: Fail
    Success --> [*]
    Error --> Idle: Retry`}},t={args:{code:`gantt
    title Project Timeline
    dateFormat YYYY-MM-DD
    section Phase 1
    Task 1 :a1, 2024-01-01, 30d
    Task 2 :a2, after a1, 20d
    section Phase 2
    Task 3 :a3, 2024-02-01, 25d`}},n={args:{code:`classDiagram
    class User {
        +String name
        +String email
        +login()
    }
    class Order {
        +Date date
        +calculateTotal()
    }
    User "1" --> "*" Order`}};e.parameters={...e.parameters,docs:{...e.parameters?.docs,source:{originalSource:`{
  args: {
    code: \`graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Action 1]
    B -->|No| D[Action 2]
    C --> E[End]
    D --> E\`
  }
}`,...e.parameters?.docs?.source}}};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  args: {
    code: \`sequenceDiagram
    participant User
    participant Web
    participant API
    participant DB
    
    User->>Web: Request
    Web->>API: API Call
    API->>DB: Query
    DB-->>API: Results
    API-->>Web: Response
    Web-->>User: Render\`
  }
}`,...a.parameters?.docs?.source}}};r.parameters={...r.parameters,docs:{...r.parameters?.docs,source:{originalSource:`{
  args: {
    code: \`graph LR
    A[User] --> B[Web Server]
    B --> C[API Gateway]
    C --> D[Service 1]
    C --> E[Service 2]
    D --> F[Database]
    E --> F\`
  }
}`,...r.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  args: {
    code: \`stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: Start
    Processing --> Success: Complete
    Processing --> Error: Fail
    Success --> [*]
    Error --> Idle: Retry\`
  }
}`,...s.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  args: {
    code: \`gantt
    title Project Timeline
    dateFormat YYYY-MM-DD
    section Phase 1
    Task 1 :a1, 2024-01-01, 30d
    Task 2 :a2, after a1, 20d
    section Phase 2
    Task 3 :a3, 2024-02-01, 25d\`
  }
}`,...t.parameters?.docs?.source}}};n.parameters={...n.parameters,docs:{...n.parameters?.docs,source:{originalSource:`{
  args: {
    code: \`classDiagram
    class User {
        +String name
        +String email
        +login()
    }
    class Order {
        +Date date
        +calculateTotal()
    }
    User "1" --> "*" Order\`
  }
}`,...n.parameters?.docs?.source}}};const g=["Flowchart","SequenceDiagram","ArchitectureDiagram","StateDiagram","GanttChart","ClassDiagram"];export{r as ArchitectureDiagram,n as ClassDiagram,e as Flowchart,t as GanttChart,a as SequenceDiagram,s as StateDiagram,g as __namedExportsOrder,p as default};
