import{j as e}from"./iframe-CLHqt8sP.js";import{B as r}from"./Badge-DGs5HKmX.js";import{C as a}from"./Card-BHe2Chf6.js";import"./preload-helper-PPVm8Dsz.js";import"./transition-Bjg5DtQF.js";import"./use-sync-refs-v6ctzwoA.js";import"./open-closed-B--VY9yg.js";const x={title:"Components/Badge",component:r,tags:["autodocs"],parameters:{docs:{description:{component:"Badge component for displaying status, labels, and metadata. Used throughout Sruja to indicate element types, validation status, and system states."}}},argTypes:{color:{control:{type:"select"},options:["default","brand","success","error","warning","info"],description:"Color variant of the badge"},children:{control:{type:"text"},description:"Badge content"}}},o={args:{children:"Badge",color:"brand"}},t={render:()=>e.jsxs("div",{style:{display:"flex",flexWrap:"wrap",gap:8},children:[e.jsx(r,{children:"Neutral"}),e.jsx(r,{color:"brand",children:"Brand"}),e.jsx(r,{color:"success",children:"Success"}),e.jsx(r,{color:"error",children:"Error"}),e.jsx(r,{color:"warning",children:"Warning"}),e.jsx(r,{color:"info",children:"Info"})]}),parameters:{docs:{description:{story:"All available badge color variants."}}}},s={render:()=>e.jsxs("div",{style:{display:"flex",flexDirection:"column",gap:12},children:[e.jsxs("div",{style:{display:"flex",gap:8,alignItems:"center"},children:[e.jsx("span",{style:{fontSize:14,color:"var(--color-text-secondary)",minWidth:100},children:"Person:"}),e.jsx(r,{color:"info",children:"User"}),e.jsx(r,{color:"info",children:"Admin"})]}),e.jsxs("div",{style:{display:"flex",gap:8,alignItems:"center"},children:[e.jsx("span",{style:{fontSize:14,color:"var(--color-text-secondary)",minWidth:100},children:"System:"}),e.jsx(r,{color:"brand",children:"Web Application"}),e.jsx(r,{color:"brand",children:"API Gateway"})]}),e.jsxs("div",{style:{display:"flex",gap:8,alignItems:"center"},children:[e.jsx("span",{style:{fontSize:14,color:"var(--color-text-secondary)",minWidth:100},children:"Container:"}),e.jsx(r,{children:"API Service"}),e.jsx(r,{children:"Web Server"})]}),e.jsxs("div",{style:{display:"flex",gap:8,alignItems:"center"},children:[e.jsx("span",{style:{fontSize:14,color:"var(--color-text-secondary)",minWidth:100},children:"Datastore:"}),e.jsx(r,{color:"warning",children:"PostgreSQL"}),e.jsx(r,{color:"warning",children:"Redis"})]})]}),parameters:{docs:{description:{story:"Badges used to label different architecture element types."}}}},i={render:()=>e.jsxs("div",{style:{display:"flex",flexDirection:"column",gap:12},children:[e.jsx(a,{title:"Validation Status",subtitle:"Architecture validation results",children:e.jsxs("div",{style:{display:"flex",gap:8,flexWrap:"wrap"},children:[e.jsx(r,{color:"success",children:"Valid"}),e.jsx(r,{color:"error",children:"2 Errors"}),e.jsx(r,{color:"warning",children:"1 Warning"})]})}),e.jsx(a,{title:"System Health",subtitle:"Component status indicators",children:e.jsxs("div",{style:{display:"flex",gap:8,flexWrap:"wrap"},children:[e.jsx(r,{color:"success",children:"Healthy"}),e.jsx(r,{color:"warning",children:"Degraded"}),e.jsx(r,{color:"error",children:"Down"})]})})]}),parameters:{docs:{description:{story:"Badges used for status indicators in architecture validation and system health."}}}},n={render:()=>e.jsxs("div",{style:{display:"grid",gridTemplateColumns:"repeat(auto-fit, minmax(250px, 1fr))",gap:16},children:[e.jsx(a,{title:"E-commerce Platform",subtitle:"Production system",footer:e.jsx(r,{color:"success",children:"Active"}),children:e.jsx("p",{style:{fontSize:14,color:"var(--color-text-secondary)",margin:0},children:"Main e-commerce application with payment processing."})}),e.jsx(a,{title:"API Gateway",subtitle:"Infrastructure component",footer:e.jsx(r,{color:"info",children:"Beta"}),children:e.jsx("p",{style:{fontSize:14,color:"var(--color-text-secondary)",margin:0},children:"API routing and load balancing service."})}),e.jsx(a,{title:"User Database",subtitle:"Data storage",footer:e.jsx(r,{color:"warning",children:"Maintenance"}),children:e.jsx("p",{style:{fontSize:14,color:"var(--color-text-secondary)",margin:0},children:"PostgreSQL database for user data."})})]}),parameters:{docs:{description:{story:"Badges used in card footers to show status or metadata."}}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  args: {
    children: 'Badge',
    color: 'brand'
  }
}`,...o.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    display: 'flex',
    flexWrap: 'wrap',
    gap: 8
  }}>
      <Badge>Neutral</Badge>
      <Badge color="brand">Brand</Badge>
      <Badge color="success">Success</Badge>
      <Badge color="error">Error</Badge>
      <Badge color="warning">Warning</Badge>
      <Badge color="info">Info</Badge>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'All available badge color variants.'
      }
    }
  }
}`,...t.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    display: 'flex',
    flexDirection: 'column',
    gap: 12
  }}>
      <div style={{
      display: 'flex',
      gap: 8,
      alignItems: 'center'
    }}>
        <span style={{
        fontSize: 14,
        color: 'var(--color-text-secondary)',
        minWidth: 100
      }}>Person:</span>
        <Badge color="info">User</Badge>
        <Badge color="info">Admin</Badge>
      </div>
      <div style={{
      display: 'flex',
      gap: 8,
      alignItems: 'center'
    }}>
        <span style={{
        fontSize: 14,
        color: 'var(--color-text-secondary)',
        minWidth: 100
      }}>System:</span>
        <Badge color="brand">Web Application</Badge>
        <Badge color="brand">API Gateway</Badge>
      </div>
      <div style={{
      display: 'flex',
      gap: 8,
      alignItems: 'center'
    }}>
        <span style={{
        fontSize: 14,
        color: 'var(--color-text-secondary)',
        minWidth: 100
      }}>Container:</span>
        <Badge>API Service</Badge>
        <Badge>Web Server</Badge>
      </div>
      <div style={{
      display: 'flex',
      gap: 8,
      alignItems: 'center'
    }}>
        <span style={{
        fontSize: 14,
        color: 'var(--color-text-secondary)',
        minWidth: 100
      }}>Datastore:</span>
        <Badge color="warning">PostgreSQL</Badge>
        <Badge color="warning">Redis</Badge>
      </div>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Badges used to label different architecture element types.'
      }
    }
  }
}`,...s.parameters?.docs?.source}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    display: 'flex',
    flexDirection: 'column',
    gap: 12
  }}>
      <Card title="Validation Status" subtitle="Architecture validation results">
        <div style={{
        display: 'flex',
        gap: 8,
        flexWrap: 'wrap'
      }}>
          <Badge color="success">Valid</Badge>
          <Badge color="error">2 Errors</Badge>
          <Badge color="warning">1 Warning</Badge>
        </div>
      </Card>
      <Card title="System Health" subtitle="Component status indicators">
        <div style={{
        display: 'flex',
        gap: 8,
        flexWrap: 'wrap'
      }}>
          <Badge color="success">Healthy</Badge>
          <Badge color="warning">Degraded</Badge>
          <Badge color="error">Down</Badge>
        </div>
      </Card>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Badges used for status indicators in architecture validation and system health.'
      }
    }
  }
}`,...i.parameters?.docs?.source}}};n.parameters={...n.parameters,docs:{...n.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
    gap: 16
  }}>
      <Card title="E-commerce Platform" subtitle="Production system" footer={<Badge color="success">Active</Badge>}>
        <p style={{
        fontSize: 14,
        color: 'var(--color-text-secondary)',
        margin: 0
      }}>
          Main e-commerce application with payment processing.
        </p>
      </Card>
      <Card title="API Gateway" subtitle="Infrastructure component" footer={<Badge color="info">Beta</Badge>}>
        <p style={{
        fontSize: 14,
        color: 'var(--color-text-secondary)',
        margin: 0
      }}>
          API routing and load balancing service.
        </p>
      </Card>
      <Card title="User Database" subtitle="Data storage" footer={<Badge color="warning">Maintenance</Badge>}>
        <p style={{
        fontSize: 14,
        color: 'var(--color-text-secondary)',
        margin: 0
      }}>
          PostgreSQL database for user data.
        </p>
      </Card>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Badges used in card footers to show status or metadata.'
      }
    }
  }
}`,...n.parameters?.docs?.source}}};const u=["Playground","Variants","ElementTypes","StatusIndicators","InCards"];export{s as ElementTypes,n as InCards,o as Playground,i as StatusIndicators,t as Variants,u as __namedExportsOrder,x as default};
