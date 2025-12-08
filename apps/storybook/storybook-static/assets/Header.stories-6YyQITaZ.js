import{j as e}from"./iframe-CLHqt8sP.js";import{H as l,D as d}from"./download-DlmzcEjP.js";import{T as t}from"./ThemeToggle-DOgjCiVE.js";import{B as n}from"./Button-DrzFMabF.js";import"./preload-helper-PPVm8Dsz.js";import"./Logo-CKDpWWdM.js";import"./cn-VBtxfiI-.js";import"./bundle-mjs-COJ8Fh6m.js";import"./createLucideIcon-Byd03hVy.js";import"./variants-DKu3G6j2.js";const S={title:"Components/Header",component:l,tags:["autodocs"],parameters:{layout:"centered",docs:{description:{component:"Application header component used in Sruja Studio. Displays branding, version information, and action buttons. Supports custom left and right content areas for flexible layouts."}}},argTypes:{title:{control:{type:"text"},description:"Main application title"},subtitle:{control:{type:"text"},description:"Subtitle or tagline"},version:{control:{type:"text"},description:"Version number displayed in header"},logoLoading:{control:{type:"boolean"},description:"Shows loading state for logo"}}},o={args:{title:"Sruja Studio",subtitle:"Architecture Visualization Tool",version:"0.1.0",leftContent:e.jsx("div",{}),rightContent:e.jsx(t,{iconOnly:!0,size:"sm"})}},r={args:{title:"Sruja Studio",subtitle:"Architecture Visualization Tool",version:"0.1.0",logoLoading:!1,logoSize:32,leftContent:e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:12},children:[e.jsx("label",{style:{fontSize:14,fontWeight:500,color:"var(--color-text-secondary)"},children:"Example:"}),e.jsxs("select",{style:{padding:"6px 12px",borderRadius:6,border:"1px solid var(--color-border)",fontSize:14,backgroundColor:"var(--color-background)",color:"var(--color-text-primary)"},children:[e.jsx("option",{children:"Simple Web App"}),e.jsx("option",{children:"E-commerce Platform"}),e.jsx("option",{children:"Microservices"})]})]}),rightContent:e.jsxs("div",{style:{display:"flex",alignItems:"center",gap:8},children:[e.jsx(n,{variant:"ghost",size:"sm",children:"Preview Markdown"}),e.jsxs(n,{size:"sm",style:{display:"flex",alignItems:"center",gap:6},children:[e.jsx(d,{size:16}),"Export"]}),e.jsx(t,{iconOnly:!0,size:"sm"})]})},parameters:{docs:{description:{story:"Header layout as used in Sruja Studio with example selector and action buttons."}}}},i={render:()=>e.jsx("div",{className:"bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-3xl",children:e.jsx(l,{title:"Sruja Studio",subtitle:"Architecture Visualization Tool",version:"0.1.0",leftContent:e.jsx("div",{}),rightContent:e.jsx(t,{iconOnly:!0,size:"sm"})})})},s={args:{title:"Sruja",subtitle:"Architecture as Code",version:"0.1.0",leftContent:e.jsx("div",{}),rightContent:e.jsx(t,{iconOnly:!0,size:"sm"})},parameters:{docs:{description:{story:"Minimal header with just branding and theme toggle."}}}},a={args:{title:"Sruja Studio",subtitle:"Loading architecture...",version:"0.1.0",logoLoading:!0,leftContent:e.jsx("div",{}),rightContent:e.jsx(t,{iconOnly:!0,size:"sm"})},parameters:{docs:{description:{story:"Header with loading state while initializing WASM or parsing architecture."}}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  args: {
    title: 'Sruja Studio',
    subtitle: 'Architecture Visualization Tool',
    version: '0.1.0',
    leftContent: <div />,
    rightContent: <ThemeToggle iconOnly size="sm" />
  }
}`,...o.parameters?.docs?.source}}};r.parameters={...r.parameters,docs:{...r.parameters?.docs,source:{originalSource:`{
  args: {
    title: 'Sruja Studio',
    subtitle: 'Architecture Visualization Tool',
    version: '0.1.0',
    logoLoading: false,
    logoSize: 32,
    leftContent: <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: 12
    }}>
        <label style={{
        fontSize: 14,
        fontWeight: 500,
        color: 'var(--color-text-secondary)'
      }}>
          Example:
        </label>
        <select style={{
        padding: '6px 12px',
        borderRadius: 6,
        border: '1px solid var(--color-border)',
        fontSize: 14,
        backgroundColor: 'var(--color-background)',
        color: 'var(--color-text-primary)'
      }}>
          <option>Simple Web App</option>
          <option>E-commerce Platform</option>
          <option>Microservices</option>
        </select>
      </div>,
    rightContent: <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: 8
    }}>
        <Button variant="ghost" size="sm">
          Preview Markdown
        </Button>
        <Button size="sm" style={{
        display: 'flex',
        alignItems: 'center',
        gap: 6
      }}>
          <Download size={16} />
          Export
        </Button>
        <ThemeToggle iconOnly size="sm" />
      </div>
  },
  parameters: {
    docs: {
      description: {
        story: 'Header layout as used in Sruja Studio with example selector and action buttons.'
      }
    }
  }
}`,...r.parameters?.docs?.source}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  render: () => <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-3xl">
      <Header title="Sruja Studio" subtitle="Architecture Visualization Tool" version="0.1.0" leftContent={<div />} rightContent={<ThemeToggle iconOnly size="sm" />} />
    </div>
}`,...i.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  args: {
    title: 'Sruja',
    subtitle: 'Architecture as Code',
    version: '0.1.0',
    leftContent: <div />,
    rightContent: <ThemeToggle iconOnly size="sm" />
  },
  parameters: {
    docs: {
      description: {
        story: 'Minimal header with just branding and theme toggle.'
      }
    }
  }
}`,...s.parameters?.docs?.source}}};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  args: {
    title: 'Sruja Studio',
    subtitle: 'Loading architecture...',
    version: '0.1.0',
    logoLoading: true,
    leftContent: <div />,
    rightContent: <ThemeToggle iconOnly size="sm" />
  },
  parameters: {
    docs: {
      description: {
        story: 'Header with loading state while initializing WASM or parsing architecture.'
      }
    }
  }
}`,...a.parameters?.docs?.source}}};const j=["Playground","StudioHeader","Showcase","Minimal","WithLoading"];export{s as Minimal,o as Playground,i as Showcase,r as StudioHeader,a as WithLoading,j as __namedExportsOrder,S as default};
