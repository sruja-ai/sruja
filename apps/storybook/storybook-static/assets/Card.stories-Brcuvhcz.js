import{j as e}from"./iframe-CLHqt8sP.js";import{C as l}from"./Card-BHe2Chf6.js";import{B as t}from"./Badge-DGs5HKmX.js";import{B as n}from"./Button-DrzFMabF.js";import"./preload-helper-PPVm8Dsz.js";import"./transition-Bjg5DtQF.js";import"./use-sync-refs-v6ctzwoA.js";import"./open-closed-B--VY9yg.js";import"./variants-DKu3G6j2.js";import"./bundle-mjs-COJ8Fh6m.js";const b={title:"Components/Card",component:l,tags:["autodocs"],parameters:{layout:"centered",docs:{description:{component:"Versatile card with header, footer, and interactive states. Designed for dashboards and rich content layouts with consistent spacing and elevation."}}},argTypes:{title:{control:{type:"text"},description:"Main title displayed in the card header"},subtitle:{control:{type:"text"},description:"Secondary text displayed below the title"},interactive:{control:{type:"boolean"},description:"Enables hover and focus states for clickable cards"},onClick:{action:"clicked",description:"Callback fired when card is clicked"}}},s={args:{title:"Card Title",subtitle:"Optional subtitle",children:"Content inside the card.",footer:e.jsx(t,{color:"brand",children:"Footer Info"}),interactive:!0}},a={args:{title:"Architecture Overview",subtitle:"System components and relationships",children:e.jsxs("div",{className:"space-y-2",children:[e.jsx("p",{className:"text-sm text-[var(--color-text-secondary)]",children:"This card displays a basic architecture overview with title, subtitle, and content."}),e.jsxs("ul",{className:"text-sm text-[var(--color-text-secondary)] list-disc list-inside space-y-1",children:[e.jsx("li",{children:"Web Application"}),e.jsx("li",{children:"API Service"}),e.jsx("li",{children:"Database"})]})]})},parameters:{docs:{description:{story:"Basic card with title, subtitle, and content area."}}}},r={args:{title:"Project Status",subtitle:"Last updated 2 hours ago",children:e.jsxs("div",{className:"space-y-2",children:[e.jsxs("div",{className:"flex items-center justify-between",children:[e.jsx("span",{className:"text-sm text-[var(--color-text-secondary)]",children:"Completion"}),e.jsx("span",{className:"text-sm font-medium text-[var(--color-text-primary)]",children:"75%"})]}),e.jsx("div",{className:"w-full bg-[var(--color-neutral-200)] rounded-full h-2",children:e.jsx("div",{className:"bg-[var(--color-primary)] h-2 rounded-full",style:{width:"75%"}})})]}),footer:e.jsxs("div",{className:"flex items-center justify-between",children:[e.jsx(t,{color:"success",children:"Active"}),e.jsx(n,{variant:"ghost",size:"sm",children:"View Details"})]})},parameters:{docs:{description:{story:"Card with footer section containing badges and actions."}}}},i={args:{title:"E-commerce Platform",subtitle:"Click to view architecture details",children:e.jsxs("div",{className:"space-y-2",children:[e.jsx("p",{className:"text-sm text-[var(--color-text-secondary)]",children:"A comprehensive e-commerce solution with payment processing, inventory management, and order fulfillment."}),e.jsxs("div",{className:"flex gap-2 flex-wrap",children:[e.jsx(t,{color:"info",children:"Web App"}),e.jsx(t,{color:"info",children:"API"}),e.jsx(t,{color:"info",children:"Database"})]})]}),footer:e.jsx(t,{color:"brand",children:"12 Components"}),interactive:!0,onClick:()=>alert("Card clicked!")},parameters:{docs:{description:{story:"Interactive card with hover and focus states. Click to trigger action."}}}},o={render:()=>e.jsxs("div",{className:"grid grid-cols-1 md:grid-cols-3 gap-4",children:[e.jsx(l,{title:"Architecture as Code",subtitle:"Define with DSL",footer:e.jsx(t,{color:"brand",children:"New"}),children:e.jsx("p",{className:"text-sm text-[var(--color-text-secondary)]",children:"Write architecture definitions using a simple, readable domain-specific language."})}),e.jsx(l,{title:"Visual Diagrams",subtitle:"Auto-generated",footer:e.jsx(t,{color:"success",children:"Active"}),children:e.jsx("p",{className:"text-sm text-[var(--color-text-secondary)]",children:"Automatically generate beautiful diagrams from your architecture code."})}),e.jsx(l,{title:"Validation Engine",subtitle:"Catch errors early",footer:e.jsx(t,{color:"info",children:"Beta"}),children:e.jsx("p",{className:"text-sm text-[var(--color-text-secondary)]",children:"Built-in validation ensures your architecture is consistent and error-free."})})]}),parameters:{docs:{description:{story:"Card grid layout for showcasing features or content collections."}}}},c={args:{title:"System Health",subtitle:"Real-time monitoring",children:e.jsxs("div",{className:"space-y-4",children:[e.jsxs("div",{className:"flex items-center justify-between",children:[e.jsx("span",{className:"text-2xl font-bold text-[var(--color-text-primary)]",children:"98.5%"}),e.jsx(t,{color:"success",children:"Healthy"})]}),e.jsxs("div",{className:"space-y-2",children:[e.jsxs("div",{className:"flex justify-between text-sm",children:[e.jsx("span",{className:"text-[var(--color-text-secondary)]",children:"Uptime"}),e.jsx("span",{className:"text-[var(--color-text-primary)] font-medium",children:"99.9%"})]}),e.jsxs("div",{className:"flex justify-between text-sm",children:[e.jsx("span",{className:"text-[var(--color-text-secondary)]",children:"Response Time"}),e.jsx("span",{className:"text-[var(--color-text-primary)] font-medium",children:"120ms"})]}),e.jsxs("div",{className:"flex justify-between text-sm",children:[e.jsx("span",{className:"text-[var(--color-text-secondary)]",children:"Active Users"}),e.jsx("span",{className:"text-[var(--color-text-primary)] font-medium",children:"1,234"})]})]})]}),footer:e.jsx(n,{variant:"ghost",size:"sm",className:"w-full",children:"View Full Report"})},parameters:{docs:{description:{story:"Dashboard-style card with metrics and statistics."}}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  args: {
    title: 'Card Title',
    subtitle: 'Optional subtitle',
    children: 'Content inside the card.',
    footer: <Badge color="brand">Footer Info</Badge>,
    interactive: true
  }
}`,...s.parameters?.docs?.source}}};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  args: {
    title: 'Architecture Overview',
    subtitle: 'System components and relationships',
    children: <div className="space-y-2">
        <p className="text-sm text-[var(--color-text-secondary)]">
          This card displays a basic architecture overview with title, subtitle, and content.
        </p>
        <ul className="text-sm text-[var(--color-text-secondary)] list-disc list-inside space-y-1">
          <li>Web Application</li>
          <li>API Service</li>
          <li>Database</li>
        </ul>
      </div>
  },
  parameters: {
    docs: {
      description: {
        story: 'Basic card with title, subtitle, and content area.'
      }
    }
  }
}`,...a.parameters?.docs?.source}}};r.parameters={...r.parameters,docs:{...r.parameters?.docs,source:{originalSource:`{
  args: {
    title: 'Project Status',
    subtitle: 'Last updated 2 hours ago',
    children: <div className="space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-sm text-[var(--color-text-secondary)]">Completion</span>
          <span className="text-sm font-medium text-[var(--color-text-primary)]">75%</span>
        </div>
        <div className="w-full bg-[var(--color-neutral-200)] rounded-full h-2">
          <div className="bg-[var(--color-primary)] h-2 rounded-full" style={{
          width: '75%'
        }} />
        </div>
      </div>,
    footer: <div className="flex items-center justify-between">
        <Badge color="success">Active</Badge>
        <Button variant="ghost" size="sm">View Details</Button>
      </div>
  },
  parameters: {
    docs: {
      description: {
        story: 'Card with footer section containing badges and actions.'
      }
    }
  }
}`,...r.parameters?.docs?.source}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  args: {
    title: 'E-commerce Platform',
    subtitle: 'Click to view architecture details',
    children: <div className="space-y-2">
        <p className="text-sm text-[var(--color-text-secondary)]">
          A comprehensive e-commerce solution with payment processing, inventory management, and order fulfillment.
        </p>
        <div className="flex gap-2 flex-wrap">
          <Badge color="info">Web App</Badge>
          <Badge color="info">API</Badge>
          <Badge color="info">Database</Badge>
        </div>
      </div>,
    footer: <Badge color="brand">12 Components</Badge>,
    interactive: true,
    onClick: () => alert('Card clicked!')
  },
  parameters: {
    docs: {
      description: {
        story: 'Interactive card with hover and focus states. Click to trigger action.'
      }
    }
  }
}`,...i.parameters?.docs?.source}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  render: () => <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      <Card title="Architecture as Code" subtitle="Define with DSL" footer={<Badge color="brand">New</Badge>}>
        <p className="text-sm text-[var(--color-text-secondary)]">
          Write architecture definitions using a simple, readable domain-specific language.
        </p>
      </Card>
      <Card title="Visual Diagrams" subtitle="Auto-generated" footer={<Badge color="success">Active</Badge>}>
        <p className="text-sm text-[var(--color-text-secondary)]">
          Automatically generate beautiful diagrams from your architecture code.
        </p>
      </Card>
      <Card title="Validation Engine" subtitle="Catch errors early" footer={<Badge color="info">Beta</Badge>}>
        <p className="text-sm text-[var(--color-text-secondary)]">
          Built-in validation ensures your architecture is consistent and error-free.
        </p>
      </Card>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Card grid layout for showcasing features or content collections.'
      }
    }
  }
}`,...o.parameters?.docs?.source}}};c.parameters={...c.parameters,docs:{...c.parameters?.docs,source:{originalSource:`{
  args: {
    title: 'System Health',
    subtitle: 'Real-time monitoring',
    children: <div className="space-y-4">
        <div className="flex items-center justify-between">
          <span className="text-2xl font-bold text-[var(--color-text-primary)]">98.5%</span>
          <Badge color="success">Healthy</Badge>
        </div>
        <div className="space-y-2">
          <div className="flex justify-between text-sm">
            <span className="text-[var(--color-text-secondary)]">Uptime</span>
            <span className="text-[var(--color-text-primary)] font-medium">99.9%</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-[var(--color-text-secondary)]">Response Time</span>
            <span className="text-[var(--color-text-primary)] font-medium">120ms</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-[var(--color-text-secondary)]">Active Users</span>
            <span className="text-[var(--color-text-primary)] font-medium">1,234</span>
          </div>
        </div>
      </div>,
    footer: <Button variant="ghost" size="sm" className="w-full">
        View Full Report
      </Button>
  },
  parameters: {
    docs: {
      description: {
        story: 'Dashboard-style card with metrics and statistics.'
      }
    }
  }
}`,...c.parameters?.docs?.source}}};const j=["Playground","Basic","WithFooter","Interactive","FeatureShowcase","DashboardCard"];export{a as Basic,c as DashboardCard,o as FeatureShowcase,i as Interactive,s as Playground,r as WithFooter,j as __namedExportsOrder,b as default};
