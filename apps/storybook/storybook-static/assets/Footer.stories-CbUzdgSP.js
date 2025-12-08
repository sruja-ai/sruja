import{j as e}from"./iframe-CLHqt8sP.js";import{F as s}from"./Footer-B5cUFjW8.js";import{B as i}from"./Badge-DGs5HKmX.js";import"./preload-helper-PPVm8Dsz.js";import"./cn-VBtxfiI-.js";import"./bundle-mjs-COJ8Fh6m.js";const g={title:"Components/Footer",component:s,tags:["autodocs"],parameters:{layout:"centered",docs:{description:{component:"Application footer component used in Sruja Studio. Displays copyright, branding, and links. Supports custom left, center, and right content areas for flexible layouts."}}},argTypes:{leftContent:{description:"Content displayed on the left side"},centerContent:{description:"Content displayed in the center"},rightContent:{description:"Content displayed on the right side"}}},r={args:{leftContent:e.jsxs("span",{children:["© ",new Date().getFullYear()," Sruja"]}),centerContent:e.jsx("span",{children:"Architecture as Code"}),rightContent:e.jsx("a",{href:"https://sruja.ai",target:"_blank",rel:"noopener noreferrer",style:{color:"inherit",textDecoration:"none"},children:"sruja.ai"})}},t={args:{leftContent:e.jsxs("span",{children:["© ",new Date().getFullYear()," Sruja"]}),centerContent:e.jsx("span",{children:"Architecture as Code"}),rightContent:e.jsx("a",{href:"https://sruja.ai",target:"_blank",rel:"noopener noreferrer",style:{color:"inherit",textDecoration:"none",display:"flex",alignItems:"center",gap:8},children:"sruja.ai"})},parameters:{docs:{description:{story:"Footer layout as used in Sruja Studio with branding and links."}}}},n={args:{leftContent:e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:12},children:[e.jsxs("span",{children:["© ",new Date().getFullYear()," Sruja"]}),e.jsx(i,{color:"success",style:{fontSize:11},children:"v0.1.0"})]}),centerContent:e.jsx("span",{children:"Architecture as Code"}),rightContent:e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:16},children:[e.jsx("a",{href:"https://sruja.ai",target:"_blank",rel:"noopener noreferrer",style:{color:"inherit",textDecoration:"none"},children:"Documentation"}),e.jsx("a",{href:"https://sruja.ai",target:"_blank",rel:"noopener noreferrer",style:{color:"inherit",textDecoration:"none"},children:"sruja.ai"})]})},parameters:{docs:{description:{story:"Footer with version badge and additional links."}}}},a={args:{leftContent:e.jsxs("span",{children:["© ",new Date().getFullYear()," Sruja"]}),centerContent:e.jsx("div",{}),rightContent:e.jsx("div",{})},parameters:{docs:{description:{story:"Minimal footer with just copyright information."}}}},o={render:()=>e.jsx("div",{className:"bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-3xl",children:e.jsx(s,{leftContent:e.jsxs("span",{children:["© ",new Date().getFullYear()," Sruja"]}),centerContent:e.jsx("span",{children:"Architecture as Code"}),rightContent:e.jsx("a",{href:"https://sruja.ai",target:"_blank",rel:"noopener noreferrer",className:"text-[var(--color-primary)]",children:"sruja.ai"})})})};r.parameters={...r.parameters,docs:{...r.parameters?.docs,source:{originalSource:`{
  args: {
    leftContent: <span>© {new Date().getFullYear()} Sruja</span>,
    centerContent: <span>Architecture as Code</span>,
    rightContent: <a href="https://sruja.ai" target="_blank" rel="noopener noreferrer" style={{
      color: 'inherit',
      textDecoration: 'none'
    }}>sruja.ai</a>
  }
}`,...r.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  args: {
    leftContent: <span>© {new Date().getFullYear()} Sruja</span>,
    centerContent: <span>Architecture as Code</span>,
    rightContent: <a href="https://sruja.ai" target="_blank" rel="noopener noreferrer" style={{
      color: 'inherit',
      textDecoration: 'none',
      display: 'flex',
      alignItems: 'center',
      gap: 8
    }}>
        sruja.ai
      </a>
  },
  parameters: {
    docs: {
      description: {
        story: 'Footer layout as used in Sruja Studio with branding and links.'
      }
    }
  }
}`,...t.parameters?.docs?.source}}};n.parameters={...n.parameters,docs:{...n.parameters?.docs,source:{originalSource:`{
  args: {
    leftContent: <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: 12
    }}>
        <span>© {new Date().getFullYear()} Sruja</span>
        <Badge color="success" style={{
        fontSize: 11
      }}>v0.1.0</Badge>
      </div>,
    centerContent: <span>Architecture as Code</span>,
    rightContent: <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: 16
    }}>
        <a href="https://sruja.ai" target="_blank" rel="noopener noreferrer" style={{
        color: 'inherit',
        textDecoration: 'none'
      }}>
          Documentation
        </a>
        <a href="https://sruja.ai" target="_blank" rel="noopener noreferrer" style={{
        color: 'inherit',
        textDecoration: 'none'
      }}>
          sruja.ai
        </a>
      </div>
  },
  parameters: {
    docs: {
      description: {
        story: 'Footer with version badge and additional links.'
      }
    }
  }
}`,...n.parameters?.docs?.source}}};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  args: {
    leftContent: <span>© {new Date().getFullYear()} Sruja</span>,
    centerContent: <div />,
    rightContent: <div />
  },
  parameters: {
    docs: {
      description: {
        story: 'Minimal footer with just copyright information.'
      }
    }
  }
}`,...a.parameters?.docs?.source}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  render: () => <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-3xl">
      <Footer leftContent={<span>© {new Date().getFullYear()} Sruja</span>} centerContent={<span>Architecture as Code</span>} rightContent={<a href="https://sruja.ai" target="_blank" rel="noopener noreferrer" className="text-[var(--color-primary)]">sruja.ai</a>} />
    </div>
}`,...o.parameters?.docs?.source}}};const m=["Playground","StudioFooter","WithStatus","Minimal","Showcase"];export{a as Minimal,r as Playground,o as Showcase,t as StudioFooter,n as WithStatus,m as __namedExportsOrder,g as default};
