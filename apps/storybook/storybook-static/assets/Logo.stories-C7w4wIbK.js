import{j as e}from"./iframe-CLHqt8sP.js";import{L as c}from"./Logo-CKDpWWdM.js";import"./preload-helper-PPVm8Dsz.js";import"./cn-VBtxfiI-.js";import"./bundle-mjs-COJ8Fh6m.js";const u={title:"Components/Logo",component:c,tags:["autodocs"],parameters:{docs:{description:{component:"The Sruja logo component. Can be displayed with or without a rotation animation for loading states."}}},argTypes:{size:{control:{type:"number",min:16,max:128,step:8},description:"Size of the logo in pixels",table:{type:{summary:"number"},defaultValue:{summary:"32"}}},isLoading:{control:{type:"boolean"},description:"Whether to show the logo with rotation animation",table:{defaultValue:{summary:"false"}}},className:{control:{type:"text"},description:"Additional CSS classes"},alt:{control:{type:"text"},description:"Alt text for accessibility",table:{defaultValue:{summary:"Sruja Logo"}}}}},r={args:{size:32}},s={args:{size:16}},a={args:{size:32}},o={args:{size:64}},t={args:{size:48,isLoading:!0}},n={render:()=>e.jsxs("div",{style:{padding:"2rem",display:"flex",alignItems:"center",gap:"0.5rem"},children:[e.jsx(c,{size:32}),e.jsx("span",{style:{fontSize:"1.5rem",fontWeight:"bold"},children:"Sruja"})]})},i={render:()=>e.jsxs("div",{style:{padding:"2rem",textAlign:"center"},children:[e.jsx(c,{size:48,isLoading:!0}),e.jsx("p",{style:{marginTop:"1rem",color:"#64748b"},children:"Loading..."})]})};r.parameters={...r.parameters,docs:{...r.parameters?.docs,source:{originalSource:`{
  args: {
    size: 32
  }
}`,...r.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  args: {
    size: 16
  }
}`,...s.parameters?.docs?.source}}};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  args: {
    size: 32
  }
}`,...a.parameters?.docs?.source}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  args: {
    size: 64
  }
}`,...o.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  args: {
    size: 48,
    isLoading: true
  }
}`,...t.parameters?.docs?.source}}};n.parameters={...n.parameters,docs:{...n.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    padding: '2rem',
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem'
  }}>
      <Logo size={32} />
      <span style={{
      fontSize: '1.5rem',
      fontWeight: 'bold'
    }}>Sruja</span>
    </div>
}`,...n.parameters?.docs?.source}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    padding: '2rem',
    textAlign: 'center'
  }}>
      <Logo size={48} isLoading />
      <p style={{
      marginTop: '1rem',
      color: '#64748b'
    }}>Loading...</p>
    </div>
}`,...i.parameters?.docs?.source}}};const y=["Default","Small","Medium","Large","WithAnimation","InContext","LoadingState"];export{r as Default,n as InContext,o as Large,i as LoadingState,a as Medium,s as Small,t as WithAnimation,y as __namedExportsOrder,u as default};
