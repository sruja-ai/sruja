import{R as h,j as e}from"./iframe-CLHqt8sP.js";import"./preload-helper-PPVm8Dsz.js";function l({size:r=48,className:g=""}){const u=h.useId(),m=`infGrad-${u}`,p=`sGrad-${u}`;return e.jsx("div",{className:`sruja-loader ${g}`,style:{width:r,height:r,display:"inline-flex",alignItems:"center",justifyContent:"center"},children:e.jsxs("svg",{width:r,height:r,viewBox:"0 0 300 300",xmlns:"http://www.w3.org/2000/svg",className:"sruja-loader-svg",children:[e.jsxs("defs",{children:[e.jsxs("linearGradient",{id:m,x1:"0%",y1:"0%",x2:"100%",y2:"100%",children:[e.jsx("stop",{offset:"0%",stopColor:"#7C3AED"}),e.jsx("stop",{offset:"100%",stopColor:"#2563EB"})]}),e.jsxs("linearGradient",{id:p,x1:"0%",y1:"0%",x2:"100%",y2:"0%",children:[e.jsx("stop",{offset:"0%",stopColor:"#DB2777"}),e.jsx("stop",{offset:"100%",stopColor:"#F472B6"})]})]}),e.jsx("g",{className:"sruja-loader-horizontal",children:e.jsx("path",{d:"M50,150 C50,100 135,100 150,150 C165,200 250,200 250,150 C250,100 165,100 150,150 C135,200 50,200 50,150",fill:"none",stroke:`url(#${m})`,strokeWidth:"15",strokeLinecap:"round"})}),e.jsx("g",{className:"sruja-loader-vertical",children:e.jsx("path",{d:"M150,50 C100,50 100,135 150,150 C200,165 200,250 150,250 C100,250 100,165 150,150 C200,135 200,50 150,50",fill:"none",stroke:`url(#${m})`,strokeWidth:"15",strokeLinecap:"round"})}),e.jsx("g",{className:"sruja-loader-s-highlight-vertical",children:e.jsx("path",{d:"M150,50 C100,50 100,135 150,150 C200,165 200,250 150,250",fill:"none",stroke:`url(#${p})`,strokeWidth:"18",strokeLinecap:"round"})}),e.jsx("g",{className:"sruja-loader-s-highlight-horizontal",children:e.jsx("path",{d:"M50,150 C50,200 135,200 150,150 C165,100 250,100 250,150",fill:"none",stroke:`url(#${p})`,strokeWidth:"18",strokeLinecap:"round"})}),e.jsx("circle",{cx:"150",cy:"150",r:"12",fill:"#2563EB",className:"sruja-loader-center"}),e.jsx("circle",{cx:"150",cy:"150",r:"10",fill:"white",className:"sruja-loader-center-inner"})]})})}l.__docgenInfo={description:"",methods:[],displayName:"SrujaLoader",props:{size:{required:!1,tsType:{name:"number"},description:"",defaultValue:{value:"48",computed:!1}},className:{required:!1,tsType:{name:"string"},description:"",defaultValue:{value:"''",computed:!1}}}};const f={title:"Components/SrujaLoader",component:l,tags:["autodocs"],parameters:{docs:{description:{component:"An animated loading spinner component featuring the Sruja logo. Used throughout the application to indicate loading states."}}},argTypes:{size:{control:{type:"number",min:16,max:128,step:8},description:"Size of the loader in pixels",table:{type:{summary:"number"},defaultValue:{summary:"48"}}},className:{control:{type:"text"},description:"Additional CSS classes",table:{type:{summary:"string"}}}}},s={args:{size:48}},a={args:{size:24}},o={args:{size:48}},n={args:{size:64}},t={args:{size:96}},i={args:{size:48,className:"border-2 border-primary rounded-lg p-4"}},d={render:()=>e.jsxs("div",{style:{padding:"2rem",textAlign:"center"},children:[e.jsx("div",{style:{marginBottom:"1rem",color:"#64748b"},children:"Loading your architecture..."}),e.jsx(l,{size:48})]})},c={render:()=>e.jsxs("div",{style:{padding:"1rem"},children:[e.jsx("p",{style:{display:"inline-block",marginRight:"0.5rem"},children:"Processing"}),e.jsx(l,{size:16})]})};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  args: {
    size: 48
  }
}`,...s.parameters?.docs?.source}}};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  args: {
    size: 24
  }
}`,...a.parameters?.docs?.source}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  args: {
    size: 48
  }
}`,...o.parameters?.docs?.source}}};n.parameters={...n.parameters,docs:{...n.parameters?.docs,source:{originalSource:`{
  args: {
    size: 64
  }
}`,...n.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  args: {
    size: 96
  }
}`,...t.parameters?.docs?.source}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  args: {
    size: 48,
    className: 'border-2 border-primary rounded-lg p-4'
  }
}`,...i.parameters?.docs?.source}}};d.parameters={...d.parameters,docs:{...d.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    padding: '2rem',
    textAlign: 'center'
  }}>
      <div style={{
      marginBottom: '1rem',
      color: '#64748b'
    }}>
        Loading your architecture...
      </div>
      <SrujaLoader size={48} />
    </div>
}`,...d.parameters?.docs?.source}}};c.parameters={...c.parameters,docs:{...c.parameters?.docs,source:{originalSource:`{
  render: () => <div style={{
    padding: '1rem'
  }}>
      <p style={{
      display: 'inline-block',
      marginRight: '0.5rem'
    }}>
        Processing
      </p>
      <SrujaLoader size={16} />
    </div>
}`,...c.parameters?.docs?.source}}};const y=["Default","Small","Medium","Large","ExtraLarge","WithCustomClass","InContext","Inline"];export{s as Default,t as ExtraLarge,d as InContext,c as Inline,n as Large,o as Medium,a as Small,i as WithCustomClass,y as __namedExportsOrder,f as default};
