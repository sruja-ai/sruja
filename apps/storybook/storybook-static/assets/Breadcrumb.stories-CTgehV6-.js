import{j as r}from"./iframe-CLHqt8sP.js";import{v as d}from"./variants-DKu3G6j2.js";import{c as P}from"./createLucideIcon-Byd03hVy.js";import{C as v}from"./chevron-right-DPD-XivR.js";import"./preload-helper-PPVm8Dsz.js";import"./bundle-mjs-COJ8Fh6m.js";const N=[["path",{d:"M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8",key:"5wwlr5"}],["path",{d:"M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",key:"r6nss1"}]],H=P("house",N);function p({items:e,onItemClick:u,onHomeClick:y,homeIcon:w,separator:g,showHome:f=!0,className:C=""}){const x="flex items-center gap-1 text-sm",h="px-2 py-1 rounded-md transition-colors cursor-pointer text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-surface)]",k="text-[var(--color-text-primary)] font-medium pointer-events-none",b="text-[var(--color-text-tertiary)]";return r.jsxs("nav",{className:d(x,C),"aria-label":"Breadcrumb",children:[f&&r.jsxs(r.Fragment,{children:[r.jsx("button",{onClick:y||(()=>u("root")),className:d(h,"flex items-center"),"aria-label":"Home",children:w||r.jsx(H,{size:16})}),e.length>0&&r.jsx("span",{className:b,children:g||r.jsx(v,{size:14})})]}),e.map((m,I)=>{const l=I===e.length-1;return r.jsxs("div",{className:"flex items-center gap-1",children:[r.jsx("button",{onClick:()=>u(m.id),className:d(h,l&&k),"aria-current":l?"page":void 0,children:m.label}),!l&&r.jsx("span",{className:b,children:g||r.jsx(v,{size:14})})]},m.id)})]})}p.__docgenInfo={description:"",methods:[],displayName:"Breadcrumb",props:{items:{required:!0,tsType:{name:"Array",elements:[{name:"BreadcrumbItem"}],raw:"BreadcrumbItem[]"},description:"Array of breadcrumb items"},onItemClick:{required:!0,tsType:{name:"signature",type:"function",raw:"(id: string) => void",signature:{arguments:[{type:{name:"string"},name:"id"}],return:{name:"void"}}},description:"Callback fired when a breadcrumb item is clicked"},onHomeClick:{required:!1,tsType:{name:"signature",type:"function",raw:"() => void",signature:{arguments:[],return:{name:"void"}}},description:"Callback fired when home/root is clicked"},homeIcon:{required:!1,tsType:{name:"ReactNode"},description:"Custom home icon"},separator:{required:!1,tsType:{name:"ReactNode"},description:"Custom separator between items"},showHome:{required:!1,tsType:{name:"boolean"},description:"Whether to show home button",defaultValue:{value:"true",computed:!1}},className:{required:!1,tsType:{name:"string"},description:"Additional CSS classes",defaultValue:{value:"''",computed:!1}}}};const T={title:"Components/Breadcrumb",component:p,tags:["autodocs"],parameters:{docs:{description:{component:"Breadcrumb navigation component for showing hierarchical navigation paths. Used in Sruja Viewer to show the current location in the architecture hierarchy (e.g., System > Container > Component)."}}},argTypes:{items:{description:"Array of breadcrumb items to display"},onItemClick:{action:"item clicked",description:"Callback fired when a breadcrumb item is clicked"},onHomeClick:{action:"home clicked",description:"Callback fired when home button is clicked"},showHome:{control:{type:"boolean"},description:"Whether to show the home button"}}},a={args:{items:[{id:"system1",label:"E-commerce Platform"},{id:"container1",label:"API Gateway"}],onItemClick:e=>console.log("Clicked:",e),showHome:!0}},t={args:{items:[{id:"ecommerce",label:"E-commerce Platform"}],onItemClick:e=>alert(`Navigate to: ${e}`),showHome:!0},parameters:{docs:{description:{story:"Breadcrumb showing navigation to a system level."}}}},o={args:{items:[{id:"ecommerce",label:"E-commerce Platform"},{id:"api",label:"API Gateway"}],onItemClick:e=>alert(`Navigate to: ${e}`),showHome:!0},parameters:{docs:{description:{story:"Breadcrumb showing navigation to a container within a system."}}}},n={args:{items:[{id:"ecommerce",label:"E-commerce Platform"},{id:"api",label:"API Gateway"},{id:"auth",label:"Authentication Service"}],onItemClick:e=>alert(`Navigate to: ${e}`),showHome:!0},parameters:{docs:{description:{story:"Breadcrumb showing deep navigation to a component level."}}}},i={args:{items:[{id:"system1",label:"Web Application"},{id:"container1",label:"API Service"}],onItemClick:e=>alert(`Navigate to: ${e}`),showHome:!1},parameters:{docs:{description:{story:"Breadcrumb without home button for simpler navigation."}}}},s={args:{items:[{id:"system1",label:"E-commerce Platform"},{id:"container1",label:"Payment Service"},{id:"component1",label:"Payment Processor"},{id:"subcomponent1",label:"Credit Card Handler"}],onItemClick:e=>alert(`Navigate to: ${e}`),showHome:!0},parameters:{docs:{description:{story:"Breadcrumb with a long navigation path showing multiple hierarchy levels."}}}},c={render:()=>r.jsxs("div",{style:{display:"flex",alignItems:"center",gap:16,padding:"12px 16px",backgroundColor:"var(--color-background)",borderBottom:"1px solid var(--color-border)"},children:[r.jsx("div",{style:{fontSize:14,fontWeight:600,color:"var(--color-text-primary)"},children:"Sruja Architecture"}),r.jsx(p,{items:[{id:"ecommerce",label:"E-commerce Platform"},{id:"api",label:"API Gateway"}],onItemClick:e=>alert(`Navigate to: ${e}`),showHome:!0})]}),parameters:{docs:{description:{story:"Breadcrumb integrated into a top bar navigation, as used in the Viewer app."}}}};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  args: {
    items: [{
      id: 'system1',
      label: 'E-commerce Platform'
    }, {
      id: 'container1',
      label: 'API Gateway'
    }],
    onItemClick: id => console.log('Clicked:', id),
    showHome: true
  }
}`,...a.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  args: {
    items: [{
      id: 'ecommerce',
      label: 'E-commerce Platform'
    }],
    onItemClick: id => alert(\`Navigate to: \${id}\`),
    showHome: true
  },
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb showing navigation to a system level.'
      }
    }
  }
}`,...t.parameters?.docs?.source}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  args: {
    items: [{
      id: 'ecommerce',
      label: 'E-commerce Platform'
    }, {
      id: 'api',
      label: 'API Gateway'
    }],
    onItemClick: id => alert(\`Navigate to: \${id}\`),
    showHome: true
  },
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb showing navigation to a container within a system.'
      }
    }
  }
}`,...o.parameters?.docs?.source}}};n.parameters={...n.parameters,docs:{...n.parameters?.docs,source:{originalSource:`{
  args: {
    items: [{
      id: 'ecommerce',
      label: 'E-commerce Platform'
    }, {
      id: 'api',
      label: 'API Gateway'
    }, {
      id: 'auth',
      label: 'Authentication Service'
    }],
    onItemClick: id => alert(\`Navigate to: \${id}\`),
    showHome: true
  },
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb showing deep navigation to a component level.'
      }
    }
  }
}`,...n.parameters?.docs?.source}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  args: {
    items: [{
      id: 'system1',
      label: 'Web Application'
    }, {
      id: 'container1',
      label: 'API Service'
    }],
    onItemClick: id => alert(\`Navigate to: \${id}\`),
    showHome: false
  },
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb without home button for simpler navigation.'
      }
    }
  }
}`,...i.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  args: {
    items: [{
      id: 'system1',
      label: 'E-commerce Platform'
    }, {
      id: 'container1',
      label: 'Payment Service'
    }, {
      id: 'component1',
      label: 'Payment Processor'
    }, {
      id: 'subcomponent1',
      label: 'Credit Card Handler'
    }],
    onItemClick: id => alert(\`Navigate to: \${id}\`),
    showHome: true
  },
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb with a long navigation path showing multiple hierarchy levels.'
      }
    }
  }
}`,...s.parameters?.docs?.source}}};c.parameters={...c.parameters,docs:{...c.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    display: 'flex',
    alignItems: 'center',
    gap: 16,
    padding: '12px 16px',
    backgroundColor: 'var(--color-background)',
    borderBottom: '1px solid var(--color-border)'
  }}>
      <div style={{
      fontSize: 14,
      fontWeight: 600,
      color: 'var(--color-text-primary)'
    }}>
        Sruja Architecture
      </div>
      <Breadcrumb items={[{
      id: 'ecommerce',
      label: 'E-commerce Platform'
    }, {
      id: 'api',
      label: 'API Gateway'
    }]} onItemClick={id => alert(\`Navigate to: \${id}\`)} showHome={true} />
    </div>,
  parameters: {
    docs: {
      description: {
        story: 'Breadcrumb integrated into a top bar navigation, as used in the Viewer app.'
      }
    }
  }
}`,...c.parameters?.docs?.source}}};const G=["Playground","SystemNavigation","ContainerNavigation","ComponentNavigation","WithoutHome","LongPath","InTopBar"];export{n as ComponentNavigation,o as ContainerNavigation,c as InTopBar,s as LongPath,a as Playground,t as SystemNavigation,i as WithoutHome,G as __namedExportsOrder,T as default};
