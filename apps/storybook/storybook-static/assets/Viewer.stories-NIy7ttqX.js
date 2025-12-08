import{j as e}from"./iframe-CLHqt8sP.js";import{S as n}from"./SrujaViewerView-CWue8X71.js";import"./preload-helper-PPVm8Dsz.js";import"./cytoscape.esm-BnkdMOzK.js";import"./elk.bundled-BsTYLubE.js";import"./logger-CfJbYVt4.js";const b={title:"Components/Viewer",component:n,tags:["autodocs"],parameters:{docs:{description:{component:"Interactive architecture diagram viewer. Renders C4 model diagrams from architecture JSON data. Supports zoom, pan, selection, and level navigation. This is the core visualization component in Sruja Studio."}}}},a={metadata:{name:"Simple Web Application",version:"1.0.0"},architecture:{systems:[{id:"WebApp",label:"Web Application",containers:[{id:"API",label:"API Service"},{id:"DB",label:"Database"}]}],persons:[{id:"User",label:"End User"}],relations:[{from:"User",to:"WebApp.API",verb:"Visits"},{from:"WebApp.API",to:"WebApp.DB",verb:"Reads/Writes"}]}},s={metadata:{name:"E-commerce Platform",version:"1.0.0"},architecture:{systems:[{id:"EcommerceSystem",label:"E-commerce Platform",containers:[{id:"WebApp",label:"Web Application"},{id:"API",label:"API Gateway"},{id:"Payment",label:"Payment Service"},{id:"Inventory",label:"Inventory Service"}],datastores:[{id:"UserDB",label:"User Database"},{id:"ProductDB",label:"Product Database"},{id:"OrderDB",label:"Order Database"}]}],persons:[{id:"Customer",label:"Customer"},{id:"Admin",label:"Administrator"}],relations:[{from:"Customer",to:"EcommerceSystem.WebApp",verb:"Browses"},{from:"Customer",to:"EcommerceSystem.WebApp",verb:"Purchases"},{from:"EcommerceSystem.WebApp",to:"EcommerceSystem.API",verb:"Requests"},{from:"EcommerceSystem.API",to:"EcommerceSystem.Payment",verb:"Processes"},{from:"EcommerceSystem.API",to:"EcommerceSystem.Inventory",verb:"Checks"},{from:"EcommerceSystem.Payment",to:"EcommerceSystem.OrderDB",verb:"Stores"},{from:"EcommerceSystem.Inventory",to:"EcommerceSystem.ProductDB",verb:"Queries"},{from:"Admin",to:"EcommerceSystem.API",verb:"Manages"}]}},r={render:()=>e.jsx("div",{style:{height:"500px",border:"1px solid var(--color-border)",borderRadius:8,overflow:"hidden"},children:e.jsx(n,{data:a})}),parameters:{docs:{description:{story:"Simple web application architecture with user, web app, API, and database."}}}},o={render:()=>e.jsx("div",{style:{height:"600px",border:"1px solid var(--color-border)",borderRadius:8,overflow:"hidden"},children:e.jsx(n,{data:s})}),parameters:{docs:{description:{story:"Complex e-commerce platform architecture with multiple services, databases, and user types."}}}},t={render:()=>e.jsxs("div",{style:{display:"flex",flexDirection:"column",height:"600px",border:"1px solid var(--color-border)",borderRadius:8,overflow:"hidden"},children:[e.jsxs("div",{style:{padding:12,borderBottom:"1px solid var(--color-border)",display:"flex",gap:8,alignItems:"center",backgroundColor:"var(--color-surface)"},children:[e.jsx("button",{style:{padding:"6px 12px",borderRadius:6,border:"1px solid var(--color-border)",fontSize:14,cursor:"pointer"},children:"Level 1"}),e.jsx("button",{style:{padding:"6px 12px",borderRadius:6,border:"1px solid var(--color-border)",fontSize:14,cursor:"pointer"},children:"Level 2"}),e.jsx("button",{style:{padding:"6px 12px",borderRadius:6,border:"1px solid var(--color-border)",fontSize:14,cursor:"pointer"},children:"Level 3"}),e.jsx("div",{style:{flex:1}}),e.jsx("span",{style:{fontSize:12,color:"var(--color-text-secondary)"},children:"Zoom: 100%"})]}),e.jsx("div",{style:{flex:1,position:"relative"},children:e.jsx(n,{data:s})})]}),parameters:{docs:{description:{story:"Viewer with level navigation controls and zoom indicator, as used in Studio."}}}};r.parameters={...r.parameters,docs:{...r.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    height: '500px',
    border: '1px solid var(--color-border)',
    borderRadius: 8,
    overflow: 'hidden'
  }}>
      <SrujaViewerView data={SIMPLE_WEB_APP} />
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Simple web application architecture with user, web app, API, and database.'
      }
    }
  }
}`,...r.parameters?.docs?.source}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    height: '600px',
    border: '1px solid var(--color-border)',
    borderRadius: 8,
    overflow: 'hidden'
  }}>
      <SrujaViewerView data={ECOMMERCE_PLATFORM} />
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Complex e-commerce platform architecture with multiple services, databases, and user types.'
      }
    }
  }
}`,...o.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    display: 'flex',
    flexDirection: 'column',
    height: '600px',
    border: '1px solid var(--color-border)',
    borderRadius: 8,
    overflow: 'hidden'
  }}>
      <div style={{
      padding: 12,
      borderBottom: '1px solid var(--color-border)',
      display: 'flex',
      gap: 8,
      alignItems: 'center',
      backgroundColor: 'var(--color-surface)'
    }}>
        <button style={{
        padding: '6px 12px',
        borderRadius: 6,
        border: '1px solid var(--color-border)',
        fontSize: 14,
        cursor: 'pointer'
      }}>
          Level 1
        </button>
        <button style={{
        padding: '6px 12px',
        borderRadius: 6,
        border: '1px solid var(--color-border)',
        fontSize: 14,
        cursor: 'pointer'
      }}>
          Level 2
        </button>
        <button style={{
        padding: '6px 12px',
        borderRadius: 6,
        border: '1px solid var(--color-border)',
        fontSize: 14,
        cursor: 'pointer'
      }}>
          Level 3
        </button>
        <div style={{
        flex: 1
      }} />
        <span style={{
        fontSize: 12,
        color: 'var(--color-text-secondary)'
      }}>Zoom: 100%</span>
      </div>
      <div style={{
      flex: 1,
      position: 'relative'
    }}>
        <SrujaViewerView data={ECOMMERCE_PLATFORM} />
      </div>
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Viewer with level navigation controls and zoom indicator, as used in Studio.'
      }
    }
  }
}`,...t.parameters?.docs?.source}}};const u=["SimpleWebApp","EcommercePlatform","WithControls"];export{o as EcommercePlatform,r as SimpleWebApp,t as WithControls,u as __namedExportsOrder,b as default};
