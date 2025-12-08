import{a as t,R as h,j as e}from"./iframe-CLHqt8sP.js";import{$ as se,a as re}from"./useFocusRing-4vPDgUpM.js";import{w as oe,e as ne}from"./use-resolve-button-type-DhPqIc8n.js";import{l as ce,b as ie,j as le,g as de}from"./form-fields-Djd7uIxW.js";import{Y as ue,K as G,y as he,p as pe,o as v,V as me}from"./use-sync-refs-v6ctzwoA.js";import{m as ge,n as be,o as T}from"./keyboard-DOqIamXD.js";import{M as fe,H as ke,a as Se,w as ve}from"./description-ezvdKXnU.js";import{Z as Ce,V as we,u as xe,N as ye}from"./label-TZAP9ERS.js";import{s as je}from"./bugs-ByA1MTr8.js";import{C as De}from"./Card-BHe2Chf6.js";import"./preload-helper-PPVm8Dsz.js";import"./index-DRowAfMf.js";import"./index-dLArjxyT.js";import"./hidden-CeoxBUgT.js";import"./transition-Bjg5DtQF.js";import"./open-closed-B--VY9yg.js";let $=t.createContext(null);$.displayName="GroupContext";let Ae=t.Fragment;function $e(a){var r;let[s,n]=t.useState(null),[l,d]=we(),[p,i]=ke(),u=t.useMemo(()=>({switch:s,setSwitch:n}),[s,n]),m={},D=a,f=G();return h.createElement(i,{name:"Switch.Description",value:p},h.createElement(d,{name:"Switch.Label",value:l,props:{htmlFor:(r=u.switch)==null?void 0:r.id,onClick(k){s&&(ge(k.currentTarget)&&k.preventDefault(),s.click(),s.focus({preventScroll:!0}))}}},h.createElement($.Provider,{value:u},f({ourProps:m,theirProps:D,slot:{},defaultTag:Ae,name:"Switch.Group"}))))}let Ee="button";function Pe(a,r){var s;let n=t.useId(),l=xe(),d=Se(),{id:p=l||`headlessui-switch-${n}`,disabled:i=d||!1,checked:u,defaultChecked:m,onChange:D,name:f,value:k,form:M,autoFocus:A=!1,...F}=a,E=t.useContext($),[R,B]=t.useState(null),V=t.useRef(null),I=he(V,r,E===null?null:E.setSwitch,B),g=ce(m),[S,b]=ie(u,D,g??!1),L=pe(),[U,P]=t.useState(!1),N=v(()=>{P(!0),b?.(!S),L.nextFrame(()=>{P(!1)})}),_=v(c=>{if(je(c.currentTarget))return c.preventDefault();c.preventDefault(),N()}),q=v(c=>{c.key===T.Space?(c.preventDefault(),N()):c.key===T.Enter&&de(c.currentTarget)}),K=v(c=>c.preventDefault()),H=ye(),O=ve(),{isFocusVisible:W,focusProps:J}=se({autoFocus:A}),{isHovered:X,hoverProps:Y}=re({isDisabled:i}),{pressed:Z,pressProps:z}=oe({disabled:i}),Q=be({checked:S,disabled:i,hover:X,focus:W,active:Z,autofocus:A,changing:U}),ee=me({id:p,ref:I,role:"switch",type:ne(a,R),tabIndex:a.tabIndex===-1?0:(s=a.tabIndex)!=null?s:0,"aria-checked":S,"aria-labelledby":H,"aria-describedby":O,disabled:i||void 0,autoFocus:A,onClick:_,onKeyUp:q,onKeyPress:K},J,Y,z),te=t.useCallback(()=>{if(g!==void 0)return b?.(g)},[b,g]),ae=G();return h.createElement(h.Fragment,null,f!=null&&h.createElement(le,{disabled:i,data:{[f]:k||"on"},overrides:{type:"checkbox",checked:S},form:M,onReset:te}),ae({ourProps:ee,theirProps:F,slot:Q,defaultTag:Ee,name:"Switch"}))}let Ne=ue(Pe),Te=$e,Ge=Ce,Me=fe,Fe=Object.assign(Ne,{Group:Te,Label:Ge,Description:Me});function o({checked:a,onChange:r,label:s}){return e.jsxs("div",{className:"flex items-center gap-2",children:[s&&e.jsx("span",{className:"text-sm text-[var(--color-text-secondary)]",children:s}),e.jsx(Fe,{checked:a,onChange:r,className:["relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[var(--color-primary)]",a?"bg-[var(--color-primary)]":"bg-[var(--color-neutral-200)]"].join(" "),children:e.jsx("span",{className:["inline-block h-4 w-4 transform rounded-full bg-[var(--color-background)] transition",a?"translate-x-6":"translate-x-1"].join(" ")})})]})}o.__docgenInfo={description:"",methods:[],displayName:"Switch",props:{checked:{required:!0,tsType:{name:"boolean"},description:""},onChange:{required:!0,tsType:{name:"signature",type:"function",raw:"(val: boolean) => void",signature:{arguments:[{type:{name:"boolean"},name:"val"}],return:{name:"void"}}},description:""},label:{required:!1,tsType:{name:"string"},description:""}}};const ze={title:"Components/Switch",component:o,tags:["autodocs"],parameters:{layout:"centered",docs:{description:{component:"Toggle switch component for boolean settings and preferences. Used in Sruja Studio for toggling features, view options, and configuration settings."}}},argTypes:{checked:{control:{type:"boolean"},description:"Whether the switch is checked"},disabled:{control:{type:"boolean"},description:"Disables the switch"},label:{control:{type:"text"},description:"Label text displayed next to the switch"}}},C={render:()=>{const[a,r]=t.useState(!1);return e.jsx(o,{checked:a,onChange:r,label:"Enable feature"})}},w={render:()=>{const[a,r]=t.useState(!1);return e.jsx(o,{checked:a,onChange:r,label:"Auto-save architecture"})},parameters:{docs:{description:{story:"Basic switch for toggling a single setting."}}}},x={render:()=>{const[a,r]=t.useState(!0),[s,n]=t.useState(!1),[l,d]=t.useState(!1),[p,i]=t.useState(!0),[u,m]=t.useState(!0);return e.jsx(De,{title:"Settings",subtitle:"Application preferences",children:e.jsxs("div",{style:{display:"flex",flexDirection:"column",gap:16},children:[e.jsx(o,{checked:a,onChange:r,label:"Auto-save architecture"}),e.jsx(o,{checked:s,onChange:n,label:"Show grid in diagram"}),e.jsx(o,{checked:l,onChange:d,label:"Dark mode"}),e.jsx(o,{checked:p,onChange:i,label:"Enable animations"}),e.jsx(o,{checked:u,onChange:m,label:"Real-time validation"})]})})},parameters:{docs:{description:{story:"Multiple switches in a settings panel for configuring Studio preferences."}}}},y={render:()=>{const[a,r]=t.useState(!1),[s,n]=t.useState(!0);return e.jsxs("div",{style:{display:"flex",flexDirection:"column",gap:16},children:[e.jsx(o,{checked:a,onChange:r,label:"Unchecked"}),e.jsx(o,{checked:s,onChange:n,label:"Checked"}),e.jsx(o,{checked:!1,onChange:()=>{},label:"Disabled (unchecked)",disabled:!0}),e.jsx(o,{checked:!0,onChange:()=>{},label:"Disabled (checked)",disabled:!0})]})},parameters:{docs:{description:{story:"All switch states: unchecked, checked, and disabled variants."}}}},j={render:()=>{const[a,r]=t.useState(!0),[s,n]=t.useState(!1),[l,d]=t.useState(!0);return e.jsx("div",{className:"bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-md",children:e.jsxs("div",{className:"space-y-3",children:[e.jsx(o,{checked:a,onChange:r,label:"Auto-save"}),e.jsx(o,{checked:s,onChange:n,label:"Show grid"}),e.jsx(o,{checked:l,onChange:d,label:"Enable animations"})]})})}};C.parameters={...C.parameters,docs:{...C.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [checked, setChecked] = useState(false);
    return <Switch checked={checked} onChange={setChecked} label="Enable feature" />;
  }
}`,...C.parameters?.docs?.source}}};w.parameters={...w.parameters,docs:{...w.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [checked, setChecked] = useState(false);
    return <Switch checked={checked} onChange={setChecked} label="Auto-save architecture" />;
  },
  parameters: {
    docs: {
      description: {
        story: 'Basic switch for toggling a single setting.'
      }
    }
  }
}`,...w.parameters?.docs?.source}}};x.parameters={...x.parameters,docs:{...x.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [autoSave, setAutoSave] = useState(true);
    const [showGrid, setShowGrid] = useState(false);
    const [darkMode, setDarkMode] = useState(false);
    const [animations, setAnimations] = useState(true);
    const [validation, setValidation] = useState(true);
    return <Card title="Settings" subtitle="Application preferences">
        <div style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 16
      }}>
          <Switch checked={autoSave} onChange={setAutoSave} label="Auto-save architecture" />
          <Switch checked={showGrid} onChange={setShowGrid} label="Show grid in diagram" />
          <Switch checked={darkMode} onChange={setDarkMode} label="Dark mode" />
          <Switch checked={animations} onChange={setAnimations} label="Enable animations" />
          <Switch checked={validation} onChange={setValidation} label="Real-time validation" />
        </div>
      </Card>;
  },
  parameters: {
    docs: {
      description: {
        story: 'Multiple switches in a settings panel for configuring Studio preferences.'
      }
    }
  }
}`,...x.parameters?.docs?.source}}};y.parameters={...y.parameters,docs:{...y.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [checked1, setChecked1] = useState(false);
    const [checked2, setChecked2] = useState(true);
    return <div style={{
      display: 'flex',
      flexDirection: 'column',
      gap: 16
    }}>
        <Switch checked={checked1} onChange={setChecked1} label="Unchecked" />
        <Switch checked={checked2} onChange={setChecked2} label="Checked" />
        <Switch checked={false} onChange={() => {}} label="Disabled (unchecked)" disabled />
        <Switch checked={true} onChange={() => {}} label="Disabled (checked)" disabled />
      </div>;
  },
  parameters: {
    docs: {
      description: {
        story: 'All switch states: unchecked, checked, and disabled variants.'
      }
    }
  }
}`,...y.parameters?.docs?.source}}};j.parameters={...j.parameters,docs:{...j.parameters?.docs,source:{originalSource:`{
  render: () => {
    const [a, setA] = useState(true);
    const [b, setB] = useState(false);
    const [c, setC] = useState(true);
    return <div className="bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg shadow-sm p-5 max-w-md">
        <div className="space-y-3">
          <Switch checked={a} onChange={setA} label="Auto-save" />
          <Switch checked={b} onChange={setB} label="Show grid" />
          <Switch checked={c} onChange={setC} label="Enable animations" />
        </div>
      </div>;
  }
}`,...j.parameters?.docs?.source}}};const Qe=["Playground","Basic","SettingsPanel","States","Showcase"];export{w as Basic,C as Playground,x as SettingsPanel,j as Showcase,y as States,Qe as __namedExportsOrder,ze as default};
