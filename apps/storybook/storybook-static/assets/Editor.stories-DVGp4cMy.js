import{a as d,j as e}from"./iframe-CLHqt8sP.js";import{S as a}from"./SrujaMonacoEditor-D1OWYQxt.js";import"./preload-helper-PPVm8Dsz.js";import"./MonacoEditor-BeqEQem3.js";const h={title:"Components/Editor",component:a,tags:["autodocs"],parameters:{docs:{description:{component:"Monaco-based code editor with Sruja language support. Provides syntax highlighting, autocomplete, and validation for architecture-as-code DSL. Used in Sruja Studio for editing architecture definitions."}}}},c=`system App "My App" {
  container Web "Web Server"
  datastore DB "Database"
}
person User "User"
User -> App.Web "Visits"
App.Web -> App.DB "Reads/Writes"`,n=`system EcommerceSystem "E-commerce Platform" {
  container WebApp "Web Application"
  container API "API Gateway"
  container Payment "Payment Service"
  container Inventory "Inventory Service"
  
  datastore UserDB "User Database"
  datastore ProductDB "Product Database"
  datastore OrderDB "Order Database"
}

person Customer "Customer"
person Admin "Administrator"

Customer -> EcommerceSystem.WebApp "Browses"
Customer -> EcommerceSystem.WebApp "Purchases"
EcommerceSystem.WebApp -> EcommerceSystem.API "Requests"
EcommerceSystem.API -> EcommerceSystem.Payment "Processes"
EcommerceSystem.API -> EcommerceSystem.Inventory "Checks"
EcommerceSystem.Payment -> EcommerceSystem.OrderDB "Stores"
EcommerceSystem.Inventory -> EcommerceSystem.ProductDB "Queries"
Admin -> EcommerceSystem.API "Manages"`,o={render:()=>{const[r,t]=d.useState(c);return e.jsx("div",{style:{height:"400px",border:"1px solid var(--color-border)",borderRadius:8,overflow:"hidden"},children:e.jsx(a,{value:r,onChange:t,height:"100%"})})},parameters:{docs:{description:{story:"Basic Sruja editor with simple architecture definition."}}}},i={render:()=>{const[r,t]=d.useState(n);return e.jsx("div",{style:{height:"500px",border:"1px solid var(--color-border)",borderRadius:8,overflow:"hidden"},children:e.jsx(a,{value:r,onChange:t,height:"100%"})})},parameters:{docs:{description:{story:"Editor with complex e-commerce platform architecture definition."}}}},s={render:()=>{const[r,t]=d.useState(c);return e.jsxs("div",{style:{display:"flex",height:"500px",border:"1px solid var(--color-border)",borderRadius:8,overflow:"hidden"},children:[e.jsxs("div",{style:{width:"50%",borderRight:"1px solid var(--color-border)"},children:[e.jsx("div",{style:{padding:8,borderBottom:"1px solid var(--color-border)",fontSize:12,fontWeight:600,backgroundColor:"var(--color-surface)"},children:"Editor"}),e.jsx("div",{style:{height:"calc(100% - 40px)"},children:e.jsx(a,{value:r,onChange:t,height:"100%"})})]}),e.jsx("div",{style:{width:"50%",backgroundColor:"var(--color-surface)",display:"flex",alignItems:"center",justifyContent:"center",color:"var(--color-text-secondary)"},children:e.jsxs("div",{style:{textAlign:"center"},children:[e.jsx("div",{style:{fontSize:18,fontWeight:600,marginBottom:8},children:"Diagram Preview"}),e.jsx("div",{style:{fontSize:14},children:"Interactive viewer would appear here"})]})})]})},parameters:{docs:{description:{story:"Split view layout with editor on left and diagram preview on right, as used in Studio."}}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [value, setValue] = useState(SIMPLE_EXAMPLE);
    return <div style={{
      height: '400px',
      border: '1px solid var(--color-border)',
      borderRadius: 8,
      overflow: 'hidden'
    }}>
        <SrujaMonacoEditor value={value} onChange={setValue} height="100%" />
      </div>;
  },
  parameters: {
    docs: {
      description: {
        story: 'Basic Sruja editor with simple architecture definition.'
      }
    }
  }
}`,...o.parameters?.docs?.source}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [value, setValue] = useState(ECOMMERCE_EXAMPLE);
    return <div style={{
      height: '500px',
      border: '1px solid var(--color-border)',
      borderRadius: 8,
      overflow: 'hidden'
    }}>
        <SrujaMonacoEditor value={value} onChange={setValue} height="100%" />
      </div>;
  },
  parameters: {
    docs: {
      description: {
        story: 'Editor with complex e-commerce platform architecture definition.'
      }
    }
  }
}`,...i.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [value, setValue] = useState(SIMPLE_EXAMPLE);
    return <div style={{
      display: 'flex',
      height: '500px',
      border: '1px solid var(--color-border)',
      borderRadius: 8,
      overflow: 'hidden'
    }}>
        <div style={{
        width: '50%',
        borderRight: '1px solid var(--color-border)'
      }}>
          <div style={{
          padding: 8,
          borderBottom: '1px solid var(--color-border)',
          fontSize: 12,
          fontWeight: 600,
          backgroundColor: 'var(--color-surface)'
        }}>
            Editor
          </div>
          <div style={{
          height: 'calc(100% - 40px)'
        }}>
            <SrujaMonacoEditor value={value} onChange={setValue} height="100%" />
          </div>
        </div>
        <div style={{
        width: '50%',
        backgroundColor: 'var(--color-surface)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        color: 'var(--color-text-secondary)'
      }}>
          <div style={{
          textAlign: 'center'
        }}>
            <div style={{
            fontSize: 18,
            fontWeight: 600,
            marginBottom: 8
          }}>Diagram Preview</div>
            <div style={{
            fontSize: 14
          }}>Interactive viewer would appear here</div>
          </div>
        </div>
      </div>;
  },
  parameters: {
    docs: {
      description: {
        story: 'Split view layout with editor on left and diagram preview on right, as used in Studio.'
      }
    }
  }
}`,...s.parameters?.docs?.source}}};const v=["Basic","ComplexArchitecture","SplitView"];export{o as Basic,i as ComplexArchitecture,s as SplitView,v as __namedExportsOrder,h as default};
